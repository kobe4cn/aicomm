use axum::{
    body::Body,
    extract::multipart::MultipartError,
    response::{IntoResponse, Response},
    Json,
};

use serde::{Deserialize, Serialize};

use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("chat not found: {0}")]
    NotFound(String),
    #[error("create chat error {0}")]
    CreateChatError(String),
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),
    #[error("workspace not exists: {0}")]
    WorkSpaceNotExists(String),
    #[error("sqlx error {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("jwt error {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("http header error {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
    #[error("upload error {0}")]
    UploadFileError(#[from] MultipartError),
    #[error("create dir error {0}")]
    CreateDirError(#[from] std::io::Error),
    #[error("internal error {0}")]
    InternalError(String),
    #[error("message create error {0}")]
    MessageCreateError(String),
    #[error("Chat file error {0}")]
    ChatFileError(String),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<Body> {
        let status_code = match &self {
            AppError::ChatFileError(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::MessageCreateError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::UploadFileError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::CreateDirError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::CreateChatError(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::SqlxError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,

            AppError::JwtError(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::HttpHeaderError(_) => axum::http::StatusCode::UNAUTHORIZED,
            AppError::EmailAlreadyExists(_) => axum::http::StatusCode::CONFLICT,
            AppError::WorkSpaceNotExists(_) => axum::http::StatusCode::NOT_FOUND,
        };
        (status_code, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}
