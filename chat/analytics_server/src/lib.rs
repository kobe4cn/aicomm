mod config;
mod error;
mod events;
mod handler;
mod openapi;

pub use error::AppError;
pub use events::AnalyticsEventRow;

use anyhow::Context;
use axum::{Router, http::Method, middleware::from_fn_with_state, routing::post};
use clickhouse::Client;
pub use config::*;
use core::fmt;
use core_lib::{DecodingKey, EncodingKey, TokenVerify, User, set_layer, verify_token};
use dashmap::DashMap;
use handler::create_event_handler;
use openapi::OpenApiRouter;

use std::{ops::Deref, sync::Arc};
use tower_http::cors::{self, CorsLayer};

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
    pub(crate) client: clickhouse::Client,
    pub(crate) sessions: Arc<DashMap<String, (String, i64)>>,
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
    let api = Router::new()
        .route("/event", post(create_event_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>));
    let app = Router::new()
        .openapi()
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
        let client = Client::default()
            .with_url(&config.server.db_url)
            .with_database(&config.server.db_name)
            .with_user(&config.server.user)
            .with_password(&config.server.password);
        let dk = DecodingKey::load(&config.auth.pk).context("load dk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load ek failed")?;
        let sessions = Arc::new(DashMap::new());
        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                dk,
                ek,
                client,
                sessions,
            }),
        })
    }
}
