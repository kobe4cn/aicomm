use bot_server::{AppConfig, setup_pg_listener};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer as _,
    fmt::{self, Layer, time::LocalTime},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fmt_layer = fmt::layer().with_timer(LocalTime::rfc_3339());
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(layer)
        .with(fmt_layer)
        .init();
    let config = AppConfig::try_load()?;
    setup_pg_listener(&config).await?;
    info!("Bot server started");
    Ok(())
}
