use analytics_server::{AnalyticsEventRow, AppError};
use core_lib::{
    AnalyticsEvent, EventContext, GeoLocation, MessageSentEvent, NavigationEvent, SystemInfo,
    UserLoginEvent, analytics_event::EventType,
};

use uuid::Uuid;

use crate::{SimEvent, SimEventType, SimSession, SimulatorUser};

impl From<SimEvent> for AnalyticsEvent {
    fn from(wrapper: SimEvent) -> Self {
        let mut event = AnalyticsEvent::default();
        event.context = Some(wrapper.user.into());
        event.event_type = Some(match wrapper.event {
            SimEventType::Login(data) => EventType::UserLogin(UserLoginEvent { email: data.email }),
            SimEventType::Navigation(data) => EventType::Navigation(NavigationEvent {
                from: format!("/chats/{}", data.from),
                to: format!("/chats/{}", data.to),
            }),
            SimEventType::Message(data) => EventType::MessageSent(MessageSentEvent {
                chat_id: data.chat_id,
                r#type: data.r#type,
                size: data.size,
                total_files: data.total_files,
            }),
        });
        event
    }
}

impl From<SimulatorUser> for EventContext {
    fn from(user: SimulatorUser) -> Self {
        EventContext {
            client_id: user.client_id,
            app_version: user.app_version,
            system: Some(SystemInfo {
                os: user.system_os,
                arch: user.system_arch,
                language: user.system_language,
                timezone: user.system_timezone,
            }),
            user_id: user.user_id,
            ip_address: user.ip_address,
            user_agent: user.user_agent,
            referer: "".to_string(),
            geo: Some(GeoLocation {
                country: user.geo_country,
                region: user.geo_region,
                city: user.geo_city,
            }),
            client_ts: user.client_ts,
            server_ts: user.server_ts,
        }
    }
}

impl SimSession {
    pub async fn to_analytics_events(&self) -> Result<Vec<AnalyticsEventRow>, AppError> {
        let session_id = Uuid::new_v4().to_string();
        let duration = self.end.timestamp_millis() - self.start.timestamp_millis();
        let mut events = Vec::with_capacity(self.events.len());
        for event in &self.events {
            let sim_event = SimEvent {
                user: self.user.clone(),
                event: event.clone(),
            };
            let mut row: AnalyticsEventRow = AnalyticsEvent::from(sim_event).try_into()?;
            row.session_id = session_id.clone();
            row.client_ts = self.end.timestamp_millis();
            row.server_ts = self.end.timestamp_millis();
            row.duration = duration as u32;
            events.push(row);
        }
        Ok(events)
    }
}
