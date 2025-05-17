mod config;
mod error;
mod handlers;
mod openapi;

mod agent;

use anyhow::Context;
use core_lib::{set_layer, verify_token, DecodingKey, EncodingKey, TokenVerify, User};
use handlers::*;
use middlewares::verify_chat;
mod middlewares;
use openapi::OpenApiRouter;
use sqlx::{Executor, PgPool};

use sqlx_db_tester::TestPg;
use tokio::fs;
mod models;
use axum::{
    http::Method,
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
pub use error::{AppError, ErrorOutput};
pub use models::ChatFile;
use std::{fmt, ops::Deref, path::Path, sync::Arc};
use tower_http::cors::{self, CorsLayer};

pub use config::AppConfig;
#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}
#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) pool: sqlx::PgPool,
}
impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

pub async fn get_router(state: AppState) -> Result<Router, AppError> {
    let cors = CorsLayer::new()
        .allow_origin(cors::Any)
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::PUT,
        ])
        // allow requests from any origin
        .allow_headers(cors::Any);
    // let state = AppState::try_new(config).await?;
    let chat = Router::new()
        .route(
            "/{id}",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .post(send_message_handler)
                .delete(delete_chat_handler),
        )
        .route(
            "/{id}/agents",
            get(list_agents_handler)
                .post(create_agent_handler)
                .patch(update_agent_handler),
        )
        .route("/{id}/messages", get(list_messages_handler))
        .layer(from_fn_with_state(state.clone(), verify_chat))
        .route("/", get(list_chat_handler).post(create_chat_handler));

    let api = Router::new()
        .route("/users", get(list_chat_users_handler))
        .nest("/chats", chat)
        .route("/upload", post(upload_handler))
        .route("/files/{ws_id}/{*path}", get(download_file_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let app = Router::new()
        .openapi()
        .route("/", get(index_handler))
        .nest("/api", api)
        .layer(cors)
        .with_state(state);
    Ok(set_layer(app))
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        let user = self.dk.verify(token).context("verify token failed")?;
        Ok(user)
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        fs::create_dir_all(&config.server.base_dir)
            .await
            .context("create dir failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load dk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load ek failed")?;
        let pool = sqlx::PgPool::connect(&config.server.db_url)
            .await
            .context("connect to db failed")?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                pool,
            }),
        })
    }

    #[allow(unused)]
    pub async fn new_for_test() -> Result<(TestPg, Self), AppError> {
        let config = AppConfig::try_load().context("load config failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load dk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load ek failed")?;
        let post = config
            .server
            .db_url
            .rfind('/')
            .expect("db url format error");
        let server_url = &config.server.db_url[..post];
        // let server_url = Option::None;
        println!("server_url: {}", server_url);
        let (tdb, pool) = get_test_pool(Some(server_url.to_string())).await;
        Ok((
            tdb,
            Self {
                inner: Arc::new(AppStateInner {
                    config,
                    dk,
                    ek,
                    pool,
                }),
            },
        ))
    }
}

pub async fn get_test_pool(_url: Option<String>) -> (TestPg, PgPool) {
    // let url = match url {
    //     // Some(url) => url.to_string(),
    //     _ => "postgres://postgres:postgres@localhost:5432".to_string(),
    // };
    let url = "postgres://postgres:postgres@localhost:5432".to_string();
    let tdb = TestPg::new(url, Path::new("../migrations"));
    let pool = tdb.get_pool().await;
    // run prepared sql t0 insert test data

    let sql = include_str!("../fixtures/test.sql").split(";");
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for s in sql {
        if s.trim().is_empty() {
            continue;
        }
        ts.execute(s).await.expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    (tdb, pool)
}
