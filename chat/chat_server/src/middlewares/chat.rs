use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{AppError, AppState};
use core_lib::User;

pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();
    let user = parts.extensions.get::<User>().cloned().unwrap();
    if !state
        .is_chat_member(chat_id as i64, user.id)
        .await
        .unwrap_or_default()
    {
        let err =
            AppError::MessageCreateError("verify You are not a member of this chat".to_string());
        return err.into_response();
    }
    let req = Request::from_parts(parts, body);
    // req.extensions_mut().insert(user);
    next.run(req).await
}
