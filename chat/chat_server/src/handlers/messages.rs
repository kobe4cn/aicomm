use axum::{
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    Extension, Json,
};

use tokio::fs;

use tracing::{info, warn};

use crate::{models::ListMessages, AppError, AppState, ChatFile, ErrorOutput};
use core_lib::{Message, User};
#[utoipa::path(
    get,


    path = "/api/chats/{id}/messages",
    params(("id"=u64, Path, description="Chat ID"),ListMessages),
    responses(
        (status = 200, description = "Get Messages List", body=Vec<Message>),
        (status = 400, description = "Invalid input", body=ErrorOutput),
    ),
    security(
        (), // <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn list_messages_handler(
    Extension(_user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, id as _).await?;
    Ok(Json(messages))
}

pub(crate) async fn download_file_handler(
    Extension(user): Extension<User>,
    Path((ws_id, path)): Path<(i64, String)>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "File doesn't exist or you dont have permission".to_string(),
        ));
    }
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist ".to_string()));
    }

    let content_type = match mime_guess::from_path(&path).first_raw() {
        Some(content_type) => content_type.to_string(),
        None => Err(AppError::InternalError(
            "MIME Type couldn't be determined".to_string(),
        ))?,
    };

    let body = fs::read(path).await?;
    let mut header = HeaderMap::new();
    header.insert("CONTENT-TYPE", HeaderValue::from_str(&content_type)?);

    Ok((StatusCode::OK, header, body))
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = state.config.server.base_dir.clone();
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await? {
        let filename = field.file_name().map(|s| s.to_string());
        let data = field.bytes().await;
        match (filename, data) {
            (Some(filename), Ok(data)) => {
                let file = ChatFile::new(ws_id, &filename, &data);
                let path = file.path(&base_dir);
                if path.exists() {
                    info!("file {} already exists: {:?}", filename, path);
                } else {
                    fs::create_dir_all(path.parent().expect("file path parent should exists"))
                        .await?;
                    fs::write(&path, data).await?;
                }
                files.push(file.url());
            }
            _ => {
                warn!("failed to read field ");
                continue;
            }
        }
    }
    Ok(Json(files))
}
