mod adapters;

use std::fmt::Display;

pub use adapters::*;

use serde::{Deserialize, Serialize};

pub enum AiAdapter {
    OpenAI(OpenAIAdapter),
    Ollama(OllamaAdapter),
}

impl AiSdk for AiAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        match self {
            AiAdapter::OpenAI(adapter) => adapter.complete(messages).await,
            AiAdapter::Ollama(adapter) => adapter.complete(messages).await,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[allow(async_fn_in_trait)]
pub trait AiSdk {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String>;
}

impl Message {
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    pub fn system(content: String) -> Self {
        Self {
            role: Role::System,
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: Role::Assistant,
            content,
        }
    }

    pub fn user(content: String) -> Self {
        Self {
            role: Role::User,
            content,
        }
    }
}
