use analytics_server::{
    AppError,
    pb::{
        AnalyticsEvent, AppExitEvent, EventContext, GeoLocation, SystemInfo,
        analytics_event::EventType,
    },
};
use prost::Message;

use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), AppError> {
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
    let event_type = EventType::AppExit(AppExitEvent { exit_code: 1 });
    event.event_type = Some(event_type);
    println!("{:?}", event.event_type);
    let token = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NDczNjk4NDcsImV4cCI6MTc0OTk2MTg0NywibmJmIjoxNzQ3MzY5ODQ3LCJpc3MiOiJjaGF0X3NlcnZlciIsImF1ZCI6ImNoYXRfd2ViIiwiaWQiOjEsIndzX2lkIjoxLCJmdWxsbmFtZSI6ImtldmluIHlhbmciLCJlbWFpbCI6ImtldmluLnlhbmcueGd6QGdtYWlsLmNvbSIsImlzX2JvdCI6ZmFsc2UsImNyZWF0ZWRfYXQiOiIyMDI1LTA1LTA4VDA5OjE2OjMxLjczNzU0MVoifQ.8n0ME4MQ8Wv9yGHaQeJGRZ7WszMbdcjv_OLh406HNvkU2PPHhRMRtjFrCk2xMKhWuHX1tXB4zxJ4YCtPsahVDA";

    let client = Client::new();
    let response = client
        .post("http://localhost:6690/api/event")
        .header("Authorization", format!("Bearer {}", token))
        .body(event.encode_to_vec())
        .send()
        .await?;
    println!("{:?}", response);
    Ok(())
}
