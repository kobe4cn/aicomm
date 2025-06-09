use anyhow::Result;
use chrono::Utc;
use clickhouse::Client;

use fake::faker::chrono::zh_cn::DateTimeBefore;
use fake::{Fake, Faker};
use simulator::{
    AppConfig, LoginData, MessageData, NavigationData, SimEventType, SimSession, SimulatorUser,
};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    let events: Vec<SimEventType> = (0..5).map(|_| Faker.fake::<SimEventType>()).collect();
    info!("events: {:?}", events);
    let fmt_layer = fmt::layer().with_timer(LocalTime::rfc_3339());
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(layer)
        .with(fmt_layer)
        .init();

    let config = AppConfig::try_load()?;
    let users = sim_users(1000);
    let login = Faker.fake::<LoginData>();
    let navigation_data = Faker.fake::<NavigationData>();
    let message_data = Faker.fake::<MessageData>();
    let client = Client::default()
        .with_url(&config.server.db_url)
        .with_database(&config.server.db_name)
        .with_user(&config.server.user)
        .with_password(&config.server.password);

    let mut insert = client.insert("analytics_events")?;
    for user in users {
        let rows = SimSession {
            user: user.clone(),
            start: DateTimeBefore(Utc::now()).fake(),
            end: Utc::now(),
            events: vec![
                SimEventType::Login(login.clone()),
                SimEventType::Navigation(navigation_data.clone()),
                SimEventType::Message(message_data.clone()),
            ],
        }
        .to_analytics_events()
        .await?;
        for row in rows {
            info!(
                "user: {:?}, session: {:?}, event_type: {:?}",
                user.user_id, row.session_id, row.event_type
            );
            insert.write(&row).await?;
        }
    }
    insert.end().await?;
    Ok(())
}
fn sim_users(size: usize) -> Vec<SimulatorUser> {
    (0..size).map(|_| Faker.fake::<SimulatorUser>()).collect()
}
