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
    #[serde(default)]
    pub is_bot: bool,
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

#[allow(async_fn_in_trait)]
pub trait Agent {
    async fn process(&self, msg: &str, ctx: &AgentContext) -> Result<AgentDecision, AgentError>;
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub chat_id: i64,
    pub user_id: i64,
    pub workspace_id: i64,
}

impl AgentContext {
    pub fn new(chat_id: i64, user_id: i64, workspace_id: i64) -> Self {
        Self {
            chat_id,
            user_id,
            workspace_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentDecision {
    Modify(String),
    Reply(String),
    Delete,
    None,
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent not found")]
    NotFound,
    #[error("Agent error: {0}")]
    Error(String),
    #[error("Agent complete error: {0}")]
    CompleteError(String),
    #[error("{0}")]
    AnyError(#[from] anyhow::Error),
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

#[derive(
    Debug, Clone, Default, Deserialize, Serialize, PartialEq, sqlx::Type, PartialOrd, ToSchema,
)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all(serialize = "camelCase"))]
pub enum ChatType {
    #[default]
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

/*
CREATE TABLE IF NOT EXISTS chat_agents (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    type agent_type NOT NULL DEFAULT 'reply',
    prompt TEXT NOT NULL,
    args JSONB NOT NULL DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (chat_id, agent_id)
);
*/

#[derive(
    Debug, Clone, Default, Deserialize, Serialize, PartialEq, sqlx::Type, PartialOrd, ToSchema,
)]
#[sqlx(type_name = "agent_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    #[serde(alias = "proxy", alias = "Proxy")]
    #[default]
    Proxy,
    #[serde(alias = "reply", alias = "Reply")]
    Reply,
    #[serde(alias = "tap", alias = "Tap")]
    Tap,
}

#[derive(
    Debug, Clone, Default, Deserialize, Serialize, PartialEq, sqlx::Type, PartialOrd, ToSchema,
)]
#[sqlx(type_name = "adapter_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdapterType {
    #[serde(alias = "openai", alias = "OpenAI")]
    Openai,
    #[serde(alias = "ollama", alias = "Ollama")]
    #[default]
    Ollama,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow, PartialEq, ToSchema)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ChatAgents {
    pub id: i64,
    #[serde(alias = "chatId")]
    pub chat_id: i64,
    pub name: String,
    pub r#type: AgentType,
    pub prompt: String,
    pub adapter: AdapterType,
    pub model: String,
    pub args: serde_json::Value,
    #[serde(alias = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(alias = "updatedAt")]
    pub updated_at: DateTime<Utc>,
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
            is_bot: false,
        }
    }
}
