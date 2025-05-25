use analytics_server::{AppConfig, AppState, get_router};
use anyhow::Result;

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<()> {
    let fmt_layer = fmt::layer().with_timer(LocalTime::rfc_3339());
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(layer)
        .with(fmt_layer)
        .init();
    let config = AppConfig::try_load()?;
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let state = AppState::try_new(config).await?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server is listening on {})", addr);
    let route = get_router(state).await?;

    axum::serve(listener, route.into_make_service()).await?;
    Ok(())
}
