use std::{ops::Deref, sync::Arc};
mod config;
mod error;
mod notify;

use axum::{
    middleware::from_fn_with_state,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
mod sse;
pub use config::AppConfig;
use core_lib::{verify_token, DecodingKey, TokenVerify, User};
use dashmap::DashMap;
use error::AppError;
pub use notify::setup_pg_listener;

use notify::AppEvent;

use sse::sse_handler;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;
pub struct AppStateInner {
    pub config: AppConfig,
    users: UserMap,
    dk: DecodingKey,
}
impl TokenVerify for AppState {
    type Error = AppError;
    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        Ok(self.dk.verify(token)?)
    }
}
impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AppState {
    pub fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk)?;
        let users = Arc::new(DashMap::new());
        Ok(Self(Arc::new(AppStateInner { config, dk, users })))
    }
}

const INDEX_HTML: &str = include_str!("../index.html");
pub async fn get_router(config: AppConfig) -> anyhow::Result<Router> {
    let state = AppState::try_new(config).expect("app state init failed");
    setup_pg_listener(state.clone()).await?;
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
        .route("/", get(index_handler))
        .with_state(state);
    Ok(router)
}

pub async fn index_handler() -> impl IntoResponse {
    //
    Html(INDEX_HTML)
}
