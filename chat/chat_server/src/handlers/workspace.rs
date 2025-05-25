use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{AppError, AppState};
use core_lib::User;

pub(crate) async fn list_chat_users_handler(
    State(state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(users)))
}
