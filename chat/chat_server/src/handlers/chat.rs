use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    models::{CreateChat, CreateMessage},
    AppError, AppState, ErrorOutput,
};
use core_lib::{Chat, User};

#[utoipa::path(
    get,

    description = "Get Chats List",
    path = "/api/chats",
    responses(
        (status = 200, description = "Get Chats List", body=Vec<Chat>)
    ),security(
        (), // <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_all_chat(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(chats)))
}
#[utoipa::path(
    post,

    description = "Create Chat",
    path = "/api/chats",
    responses(
        (status = 201, description = "Create Chat", body=Chat)
    ),
    security(
        (), // <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(input, user.ws_id as _).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[utoipa::path(
    get,


    path = "/api/chats/{id}",
    params(("id"=u64, Path, description="Chat ID")),
    responses(
        (status = 200, description = "Get Chat", body=Chat),
        (status = 404, description = "Chat Not Found", body=ErrorOutput)
    ),
    security(
        (), // <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn get_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.get_chat_by_id(id as _).await?;
    match chat {
        Some(chat) => Ok((StatusCode::OK, Json(chat))),
        None => Err(AppError::NotFound(id.to_string())),
    }
}
pub(crate) async fn update_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.update_chat(input, id as _).await?;
    Ok((StatusCode::OK, Json(chat)))
}
pub(crate) async fn delete_chat_handler(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.delete_chat(id as _).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state.create_message(input, id, user.id as _).await?;
    Ok((StatusCode::CREATED, Json(msg)))
}
