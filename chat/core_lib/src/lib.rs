use axum::async_trait;
use chrono::{DateTime, Utc};
mod middlewares;
mod utils;
use jwt_simple::reexports::thiserror;
pub use middlewares::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
pub use thiserror::Error;
pub use utils::{DecodingKey, EncodingKey};
use utoipa::ToSchema;
#[derive(Debug, Clone, FromRow, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[serde(skip)]
    #[sqlx(default)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

#[derive(Debug, Clone, FromRow, Deserialize, Serialize, PartialEq, ToSchema)]
pub struct WorkSpace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}
#[async_trait]
pub trait Agent {
    async fn process(&self, msg: Message, ctx: &AgentContext) -> Result<AgentDecision, AgentError>;
}

#[derive(Debug, Clone)]
pub struct AgentContext {}

pub enum AgentDecision {
    Modify(String),
    Reply(String),
    Delete,
    None,
}

#[derive(Debug, Clone, Error)]
pub enum AgentError {
    #[error("Agent not found")]
    NotFound,
    #[error("Agent error: {0}")]
    Error(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, ToSchema)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    #[sqlx(skip)]
    pub agents: Vec<i64>,
    #[serde(alias = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, sqlx::Type, PartialOrd, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, ToSchema)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub modified_content: Option<String>,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}
