mod agents;
mod chat;
mod file;
mod message;
mod user;
mod workspace;

pub use agents::{CreateAgent, UpdateAgent};
pub use chat::CreateChat;
pub use message::{CreateMessage, ListMessages};
use serde::{Deserialize, Serialize};
pub use user::{CreateUser, SigninUser};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatFile {
    pub ws_id: i64,
    pub hash: String,
    pub ext: String,
}
