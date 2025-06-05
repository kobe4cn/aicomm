mod config;
mod convert;
mod faker;
use chrono::DateTime;
use chrono::Utc;
pub use config::AppConfig;
use fake::faker::address::en::CityName;
use fake::faker::address::en::CountryName;
use fake::faker::address::en::{StateName, TimeZone};
use fake::faker::internet::en::IPv4;
use fake::faker::internet::en::SafeEmail;
use fake::faker::internet::en::UserAgent;
use fake::{Dummy, uuid::UUIDv4};
use faker::*;

#[derive(Debug, Dummy, Clone)]
pub struct SimulatorUser {
    #[dummy(faker = "UUIDv4")]
    pub client_id: String,
    #[dummy(faker = "AppVersion")]
    pub app_version: String,
    #[dummy(faker = "SystemOs")]
    pub system_os: String,
    #[dummy(faker = "SystemArch")]
    pub system_arch: String,
    #[dummy(faker = "SystemLanguage")]
    pub system_language: String,
    #[dummy(faker = "TimeZone()")]
    pub system_timezone: String,
    #[dummy(faker = "UUIDv4")]
    pub user_id: String,
    #[dummy(faker = "IPv4()")]
    pub ip_address: String,
    #[dummy(faker = "UserAgent()")]
    pub user_agent: String,
    #[dummy(faker = "CountryName()")]
    pub geo_country: String,
    #[dummy(faker = "StateName()")]
    pub geo_region: String,
    #[dummy(faker = "CityName()")]
    pub geo_city: String,

    pub client_ts: i64, // DateTime64(3)
    pub server_ts: i64, // DateTime64(3)
}

#[derive(Debug, Dummy, Clone)]
pub struct LoginData {
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
}

#[derive(Debug, Dummy, Clone)]
pub struct NavigationData {
    #[dummy(faker = "1..=100")]
    pub from: String,
    #[dummy(faker = "1..=100")]
    pub to: String,
}

#[derive(Debug, Dummy, Clone)]
pub struct MessageData {
    #[dummy(faker = "UUIDv4")]
    pub chat_id: String,
    #[dummy(faker = "MessageType")]
    pub r#type: String,
    #[dummy(faker = "1..=1000")]
    pub size: i32,
    #[dummy(faker = "0..=10")]
    pub total_files: i32,
}

#[derive(Debug, Clone)]
pub struct SimEvent {
    pub user: SimulatorUser,
    pub event: SimEventType,
}

#[derive(Debug, Clone)]
pub enum SimEventType {
    Login(LoginData),
    Navigation(NavigationData),
    Message(MessageData),
}

#[derive(Debug, Clone)]

pub struct SimSession {
    pub user: SimulatorUser,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub events: Vec<SimEventType>,
}
