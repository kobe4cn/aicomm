use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("clickhouse error {0}")]
    ClickhouseError(#[from] clickhouse::error::Error),
    #[error("jwt error {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("invalid event")]
    InvalidEvent,
    #[error("create analytics event error {0}")]
    CreateAnalyticsEventError(String),
    #[error("Missing event type")]
    MissingEventType,
    #[error("Missing event context")]
    MissingEventContext,
    #[error("reqwest error {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("missing system info {0}")]
    MissingSystemInfo(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match self {
            Self::ClickhouseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::JwtError(_) => StatusCode::UNAUTHORIZED,
            Self::InvalidEvent => StatusCode::BAD_REQUEST,
            Self::CreateAnalyticsEventError(_) => StatusCode::BAD_REQUEST,
            Self::MissingEventType => StatusCode::BAD_REQUEST,
            Self::MissingEventContext => StatusCode::BAD_REQUEST,
            Self::ReqwestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::MissingSystemInfo(_) => StatusCode::BAD_REQUEST,
        };
        let msg = self.to_string();
        warn!("Status code: {}, Error: {}   ", status_code, msg);
        (status_code, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
