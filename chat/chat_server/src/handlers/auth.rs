use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use utoipa::ToSchema;

use crate::{
    models::{CreateUser, SigninUser},
    AppError, AppState, ErrorOutput,
};
#[derive(Debug, serde::Serialize, ToSchema, serde::Deserialize)]
pub struct AuthOutput {
    pub token: String,
}
#[utoipa::path(
    post,
    path = "/api/signup",
    responses(
        (status = 200, description = "User created", body = AuthOutput)
    )
)]
pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let mut header = HeaderMap::new();
    header.insert("X-Token", token.parse()?);
    Ok((StatusCode::CREATED, header))
}

#[utoipa::path(
    post,
    path = "/api/signin",
    responses(
        (status = 200, description = "User signed in", body = AuthOutput,

        examples(("Demo" = (value = json!({"email":"kevin.yang.xgz@gmail.com","password":"test123456"})))),

        )
    )

)]
pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    // info!("signin: {:?}", input);
    let user = state.verify_user(&input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(json!({"token": token}))).into_response())
        }
        None => {
            let body = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}
