use crate::{AppError, AppState, events::AnalyticsEventRow};

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, request::Parts},
    response::IntoResponse,
};
use axum_extra::protobuf::Protobuf;
use core_lib::{AnalyticsEvent, User};

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
    State(state): State<AppState>,
    parts: Parts,
    headers: HeaderMap,
    Protobuf(event): Protobuf<AnalyticsEvent>,
) -> Result<impl IntoResponse, AppError> {
    let client = state.client.clone();
    let mut row = AnalyticsEventRow::try_from(event)?;
    row.set_session_id(&state).await?;
    if let Some(country) = headers.get("X-Country") {
        if let Ok(country_str) = country.to_str() {
            row.geo_country = Some(country_str.to_string());
        }
    }
    if let Some(region) = headers.get("X-Region") {
        if let Ok(region_str) = region.to_str() {
            row.geo_region = Some(region_str.to_string());
        }
    }
    if let Some(city) = headers.get("X-City") {
        if let Ok(city_str) = city.to_str() {
            row.geo_city = Some(city_str.to_string());
        }
    }

    if let Some(user_id) = parts.extensions.get::<User>() {
        row.user_id = Some(user_id.id.to_string());
    } else {
        row.user_id = None;
    }

    // info!("row: {:?}", row);
    let mut insert = client.insert("analytics_events")?;

    insert.write(&row).await?;
    insert.end().await?;
    Ok((StatusCode::CREATED, "insert success"))
}

#[cfg(test)]
mod tests {

    use prost::Message;
    use reqwest::Client;

    use core_lib::{
        AppExitEvent, EventContext, GeoLocation, SystemInfo, analytics_event::EventType,
    };

    use super::*;
    #[ignore]
    #[tokio::test]
    async fn test_create_event_handler() -> Result<(), AppError> {
        let mut event = AnalyticsEvent::default();
        let context = EventContext {
            client_id: "123".to_string(),
            app_version: "1.0.0".to_string(),
            system: Some(SystemInfo {
                os: "macos".to_string(),
                arch: "arm64".to_string(),
                language: "chinese".to_string(),
                timezone: "Asia/Shanghai".to_string(),
            }),
            geo: Some(GeoLocation {
                country: "china".to_string(),
                region: "shanghai".to_string(),
                city: "shanghai".to_string(),
            }),
            user_id: "2".to_string(),
            ip_address: "127.0.0.1".to_string(),
            user_agent: "safari".to_string(),
            referer: "https://www.google.com".to_string(),
            client_ts: 1715904000000,
            server_ts: 1715904000000,
        };
        event.context = Some(context);
        let event_type = EventType::AppExit(AppExitEvent { exit_code: 1 });
        event.event_type = Some(event_type);
        // println!("{:?}", event.event_type);
        let token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NDczNjk4NDcsImV4cCI6MTc0OTk2MTg0NywibmJmIjoxNzQ3MzY5ODQ3LCJpc3MiOiJjaGF0X3NlcnZlciIsImF1ZCI6ImNoYXRfd2ViIiwiaWQiOjEsIndzX2lkIjoxLCJmdWxsbmFtZSI6ImtldmluIHlhbmciLCJlbWFpbCI6ImtldmluLnlhbmcueGd6QGdtYWlsLmNvbSIsImlzX2JvdCI6ZmFsc2UsImNyZWF0ZWRfYXQiOiIyMDI1LTA1LTA4VDA5OjE2OjMxLjczNzU0MVoifQ.8n0ME4MQ8Wv9yGHaQeJGRZ7WszMbdcjv_OLh406HNvkU2PPHhRMRtjFrCk2xMKhWuHX1tXB4zxJ4YCtPsahVDA";

        let client = Client::new();
        let response = client
            .post("http://localhost:6690/api/event")
            .header("Authorization", format!("Bearer {}", token))
            .body(event.encode_to_vec())
            .send()
            .await?;
        assert!(response.status().is_success());
        Ok(())
    }
}
