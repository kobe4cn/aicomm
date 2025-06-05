use clickhouse::Row;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use utoipa::ToSchema;

use crate::{AppState, error::AppError};
use core_lib::{
    AnalyticsEvent, AppExitEvent, AppStartEvent, ChatCreatedEvent, ChatJoinedEvent, ChatLeftEvent,
    EventContext, MessageSentEvent, NavigationEvent, UserLoginEvent, UserLogoutEvent,
    UserRegisterEvent, analytics_event::EventType,
};
const SESSION_TIMEOUT: i64 = 60 * 10 * 1000;
impl AnalyticsEventRow {
    pub async fn set_session_id(&mut self, state: &AppState) -> Result<(), AppError> {
        if let Some(mut ref_data) = state.sessions.get_mut(&self.client_id) {
            let (session_id, last_ts) = ref_data.value_mut();
            let duration = self.server_ts - *last_ts;
            if duration > SESSION_TIMEOUT {
                let new_session_id = uuid::Uuid::now_v7().to_string();
                *session_id = new_session_id.clone();
                *last_ts = self.server_ts;
                self.session_id = new_session_id;
                self.duration = duration as u32;
            } else {
                *last_ts = self.server_ts;
                self.session_id = session_id.clone();
                self.duration = duration as u32;
            }
        } else {
            let mut rows = state.client.query("select session_id,server_ts from analytics_events where client_id = ? order by server_ts desc limit 1")
            .bind(&self.client_id)
            .fetch::<(String,i64)>()?;
            if let Some(item) = rows.next().await? {
                let (session_id, server_ts) = item;
                let mut session_id = session_id.clone();
                let duration = self.server_ts - server_ts;
                if duration > SESSION_TIMEOUT {
                    session_id = uuid::Uuid::now_v7().to_string();
                }
                self.session_id = session_id.to_string();
                self.duration = duration as u32;
                state
                    .sessions
                    .insert(self.client_id.clone(), (session_id, self.server_ts));
            } else {
                let session_id = uuid::Uuid::now_v7().to_string();
                state
                    .sessions
                    .insert(self.client_id.clone(), (session_id.clone(), self.server_ts));
                self.session_id = session_id;
                self.duration = 0;
            }
        }
        Ok(())
    }
}

#[derive(Row, Serialize, Debug, Deserialize, ToSchema, Default)]
pub struct AnalyticsEventRow {
    // EventContext
    pub client_id: String,
    pub session_id: String,
    pub duration: u32,
    pub app_version: String,
    pub system_os: String,
    pub system_arch: String,
    pub system_language: String,
    pub system_timezone: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub geo_country: Option<String>,
    pub geo_region: Option<String>,
    pub geo_city: Option<String>,
    pub client_ts: i64, // DateTime64(3)
    pub server_ts: i64, // DateTime64(3)

    // Event type
    pub event_type: EventTypeRow,

    // AppExitEvent
    pub app_exit_code: Option<AppExitCode>,

    // UserLoginEvent/UserLogoutEvent/UserRegisterEvent
    pub email: Option<String>,

    // UserRegisterEvent
    pub register_workspace_id: Option<String>,

    // ChatCreatedEvent
    pub chat_created_workspace_id: Option<String>,

    // MessageSentEvent
    pub message_sent_chat_id: Option<String>,
    pub message_sent_type: Option<String>,
    pub message_sent_size: Option<i32>,
    pub message_sent_total_files: Option<i32>,

    // ChatJoinedEvent/ChatLeftEvent
    pub chat_joined_id: Option<String>,
    pub chat_left_id: Option<String>,

    // NavigationEvent
    pub navigation_from: Option<String>,
    pub navigation_to: Option<String>,
}
///对于clickhouse的 enum 类型，需要使用 Serialize_repr 和 Deserialize_repr 来序列化和反序列化
/// 需要cargo add serde_repr,
/// 需要repr(i8) 来指定枚举的类型
#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, ToSchema, Default)]
#[repr(i8)]
#[serde(rename_all = "snake_case")]
pub enum EventTypeRow {
    AppStart = 1,
    AppExit = 2,
    UserLogin = 3,
    UserLogout = 4,
    UserRegister = 5,
    MessageSent = 6,
    ChatCreated = 7,
    ChatJoined = 8,
    ChatLeft = 9,
    Navigation = 10,
    #[default]
    Unspecified = 0,
}

#[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr, ToSchema, Default)]
#[repr(i8)]
#[allow(dead_code)]
#[serde(rename_all = "snake_case")]
pub enum AppExitCode {
    #[default]
    Unknown = 0,
    Success = 1,
    Failure = 2,
}

impl TryFrom<AnalyticsEvent> for AnalyticsEventRow {
    type Error = AppError;
    fn try_from(event: AnalyticsEvent) -> Result<Self, Self::Error> {
        let mut row = AnalyticsEventRow::default();
        match event.event_type {
            Some(event) => event.consume(&mut row)?,
            None => return Err(AppError::MissingEventType),
        }
        match event.context {
            Some(context) => context.consume(&mut row)?,
            None => return Err(AppError::MissingEventContext),
        }
        Ok(row)
    }
}

trait EventConsume {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError>;
}

impl EventConsume for EventContext {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.client_id = self.client_id;
        row.app_version = self.app_version;
        if let Some(system) = self.system {
            row.system_os = system.os;
            row.system_arch = system.arch;
            row.system_language = system.language;
            row.system_timezone = system.timezone;
        } else {
            return Err(AppError::MissingSystemInfo(
                "system_info missing".to_string(),
            ));
        }
        if let Some(geo) = self.geo {
            row.geo_country = Some(geo.country);
            row.geo_region = Some(geo.region);
            row.geo_city = Some(geo.city);
        }
        if !self.user_id.is_empty() {
            row.user_id = Some(self.user_id);
        }
        if !self.ip_address.is_empty() {
            row.ip_address = Some(self.ip_address);
        }
        if !self.user_agent.is_empty() {
            row.user_agent = Some(self.user_agent);
        }
        if !self.referer.is_empty() {
            row.referer = Some(self.referer);
        }
        row.client_ts = self.client_ts;
        row.server_ts = chrono::Utc::now().timestamp_millis();
        Ok(())
    }
}

impl EventConsume for EventType {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        match self {
            EventType::AppStart(ev) => ev.consume(row),
            EventType::AppExit(ev) => ev.consume(row),
            EventType::UserLogin(ev) => ev.consume(row),
            EventType::UserLogout(ev) => ev.consume(row),
            EventType::UserRegister(ev) => ev.consume(row),
            EventType::MessageSent(ev) => ev.consume(row),
            EventType::ChatCreated(ev) => ev.consume(row),
            EventType::ChatJoined(ev) => ev.consume(row),
            EventType::ChatLeft(ev) => ev.consume(row),
            EventType::Navigation(ev) => ev.consume(row),
        }
    }
}

impl EventConsume for AppStartEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::AppStart;
        Ok(())
    }
}

impl EventConsume for AppExitEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::AppExit;
        row.app_exit_code = Some(AppExitCode::from(self.exit_code));
        Ok(())
    }
}

impl EventConsume for UserLoginEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::UserLogin;
        row.email = Some(self.email);
        Ok(())
    }
}

impl EventConsume for UserLogoutEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::UserLogout;
        row.email = Some(self.email);
        Ok(())
    }
}

impl EventConsume for UserRegisterEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::UserRegister;
        row.email = Some(self.email);
        row.register_workspace_id = Some(self.workspace_id);
        Ok(())
    }
}

impl EventConsume for MessageSentEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::MessageSent;
        row.message_sent_chat_id = Some(self.chat_id);
        row.message_sent_type = Some(self.r#type);
        row.message_sent_size = Some(self.size);
        row.message_sent_total_files = Some(self.total_files);
        Ok(())
    }
}

impl EventConsume for ChatCreatedEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::ChatCreated;
        row.chat_created_workspace_id = Some(self.workspace_id);
        Ok(())
    }
}

impl EventConsume for ChatJoinedEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::ChatJoined;
        row.chat_joined_id = Some(self.chat_id);
        Ok(())
    }
}

impl EventConsume for ChatLeftEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::ChatLeft;
        row.chat_left_id = Some(self.chat_id);
        Ok(())
    }
}

impl EventConsume for NavigationEvent {
    fn consume(self, row: &mut AnalyticsEventRow) -> Result<(), AppError> {
        row.event_type = EventTypeRow::Navigation;
        row.navigation_from = Some(self.from);
        row.navigation_to = Some(self.to);
        Ok(())
    }
}

impl From<i32> for AppExitCode {
    fn from(value: i32) -> Self {
        match value {
            0 => AppExitCode::Unknown,
            1 => AppExitCode::Success,
            2 => AppExitCode::Failure,
            _ => AppExitCode::Unknown,
        }
    }
}
