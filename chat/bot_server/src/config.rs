use std::{fs::File, path::PathBuf};

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

impl AppConfig {
    pub fn try_load() -> Result<Self> {
        // read from ./app.yml, or /etc/config/app.yml, or from env CHAT_CONFIG
        let ret = match (
            File::open("../bot_server/bot.yaml"),
            File::open("bot_server/bot.yaml"),
            File::open("/app/bot.yaml"),
            std::env::var("BOT_CONFIG"),
        ) {
            (Ok(reader), _, _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, _, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("No config file found"),
        };

        Ok(ret?)
    }
}
