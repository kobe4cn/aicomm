use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use core_lib::{ChatAgents, User};

use crate::{
    models::{CreateAgent, UpdateAgent},
    AppError, AppState,
};

#[utoipa::path(
    get,

    description = "list of agents",
    path = "/api/chats/{id}/agents",
    params(
        ("id" = u64, Path, description = "chat id")
    ),
    responses(
        (status = 200, description = "List of agents", body=Vec<ChatAgents>)
    ),security(// <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn list_agents_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let agents = state.list_agents(id, user.id as _).await?;
    Ok((StatusCode::OK, Json(agents)))
}

#[utoipa::path(
    post,
    path = "/api/chats/{id}/agents",
    responses(
        (status = 200, description = "Create Agent", body=ChatAgents)
    ),
    security(// <-- make optional authentication
        ("token" = [])
    )
)]
pub(crate) async fn create_agent_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateAgent>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state.create_agent(input, id, user.id as _).await?;
    Ok((StatusCode::OK, Json(agent)))
}

#[utoipa::path(
    patch,
    path = "/api/chats/{id}/agents/{agent_id}",
    params(
        ("id" = u64, Path, description = "chat id"),
        ("agent_id" = u64, Path, description = "agent id")
    ),
    responses(
        (status = 200, description = "Update Agent", body=ChatAgents)
    ),
    security(// <-- make optional authentication
        ("token" = [])
    )
)]

pub(crate) async fn update_agent_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateAgent>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state.update_agent(input, id, user.id as _).await?;
    Ok((StatusCode::OK, Json(agent)))
}
