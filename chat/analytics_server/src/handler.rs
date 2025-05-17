use crate::{AppError, AppState, events::AnalyticsEventRow, pb::AnalyticsEvent};

use axum::{
    extract::State,
    http::{StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use core_lib::User;
use tracing::info;

#[utoipa::path(
    post,
    description = "Create Event handler",
    path = "/api/event",
    responses(
        (status = 201, description = "Create Event",)
    ),
    security(
        (), // <-- make optional authentication
        ("token" = [])
    )

)]
pub(crate) async fn create_event_handler(
    parts: Parts,
    State(state): State<AppState>,
    Protobuf(event): Protobuf<AnalyticsEvent>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.client.clone();
    let mut row = AnalyticsEventRow::try_from(event)?;
    if let Some(user_id) = parts.extensions.get::<User>() {
        row.user_id = Some(user_id.id.to_string());
    } else {
        row.user_id = None;
    }
    info!("row: {:?}", row);
    let mut insert = client.insert("analytics_events")?;

    insert.write(&row).await?;
    insert.end().await?;
    Ok((StatusCode::CREATED, "insert success"))
}

#[cfg(test)]
mod tests {

    use crate::{
        AppConfig, AppState, get_router,
        pb::{AppStartEvent, EventContext, GeoLocation, SystemInfo, analytics_event::EventType},
    };
    use axum::body::Bytes;

    use axum_test::TestServer;
    use prost::Message;

    use super::*;
    #[ignore]
    #[tokio::test]
    async fn test_create_event_handler() -> Result<(), AppError> {
        let state = AppState::try_new(AppConfig::try_load().expect("load config failed"))
            .await
            .unwrap();
        let mut event = AnalyticsEvent::default();
        let context = EventContext {
            client_id: "test".to_string(),
            app_version: "1.0.0".to_string(),
            system: Some(SystemInfo {
                os: "test".to_string(),
                arch: "test".to_string(),
                language: "test".to_string(),
                timezone: "test".to_string(),
            }),
            geo: Some(GeoLocation {
                country: "test".to_string(),
                region: "test".to_string(),
                city: "test".to_string(),
            }),
            user_id: "2".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "safari".to_string(),
            referer: "".to_string(),
            client_ts: 1715904000,
            server_ts: 1715904000,
        };
        event.context = Some(context);
        let event_type = EventType::AppStart(AppStartEvent::default());
        event.event_type = Some(event_type);
        let router = get_router(state.clone()).await?;
        let client = TestServer::new(router)?;

        let token = state.ek.sign(User {
            id: 2,
            ws_id: 1,
            fullname: "kevin".to_string(),
            email: "kevin.yang.xgz@gmail.com".to_string(),
            password_hash: Some("test123456".to_string()),
            is_bot: false,
            created_at: chrono::Utc::now(),
        });
        let result = client
            .post("/api/event")
            .add_header("Authorization", format!("Bearer {}", token.unwrap()))
            .bytes(Bytes::from(event.encode_to_vec()))
            .await;
        assert!(result.status_code().is_success());
        Ok(())
    }
}
