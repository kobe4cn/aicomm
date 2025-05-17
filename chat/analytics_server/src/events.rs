use clickhouse::Row;
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    pb::{
        AnalyticsEvent, AppExitEvent, AppStartEvent, ChatCreatedEvent, ChatJoinedEvent,
        ChatLeftEvent, EventContext, MessageSentEvent, NavigationEvent, UserLoginEvent,
        UserLogoutEvent, UserRegisterEvent, analytics_event::EventType,
    },
};

#[derive(Row, Default, Serialize, Debug, Deserialize)]
pub struct AnalyticsEventRow {
    // EventContext
    pub client_id: String,
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

#[derive(Serialize, Debug, Clone, Copy, Deserialize, Default)]
#[allow(dead_code)]
#[serde(rename_all = "snake_case")]
pub enum EventTypeRow {
    AppStart,
    AppExit,
    UserLogin,
    UserLogout,
    UserRegister,
    MessageSent,
    ChatCreated,
    ChatJoined,
    ChatLeft,
    Navigation,
    #[default]
    Unspecified,
}

#[derive(Serialize, Debug, Clone, Copy, Deserialize, Default)]
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
        row.server_ts = self.server_ts;
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
