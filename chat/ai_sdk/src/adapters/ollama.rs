use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AiAdapter, AiSdk, Message};

pub struct OllamaAdapter {
    host: String,
    model: String,
    client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}
#[derive(Serialize)]
pub struct OllamaCompletionRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
}

#[derive(Deserialize)]
pub struct OllamaCompletionResponse {
    pub model: String,
    pub created_at: String,
    pub message: OllamaMessage,
    pub done: bool,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u64,
    pub prompt_eval_duration: u64,
    pub eval_count: u64,
    pub eval_duration: u64,
}

impl OllamaAdapter {
    pub fn new(host: String, model: String) -> Self {
        Self {
            host,
            model,
            client: Client::new(),
        }
    }
    pub fn new_local(model: String) -> Self {
        Self::new("http://localhost:11434".to_string(), model)
    }
}

impl From<OllamaAdapter> for AiAdapter {
    fn from(adapter: OllamaAdapter) -> Self {
        AiAdapter::Ollama(adapter)
    }
}

impl Default for OllamaAdapter {
    fn default() -> Self {
        Self::new_local("llama3.2".to_string())
    }
}

impl AiSdk for OllamaAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let request = OllamaCompletionRequest {
            model: self.model.clone(),
            messages: messages.iter().map(|m| m.into()).collect(),
            stream: false,
        };
        let response = self
            .client
            .post(format!("{}/api/chat", self.host))
            .json(&request)
            .send()
            .await?;
        let data: OllamaCompletionResponse = response.json().await?;
        let content = data.message.content;
        Ok(content)
    }
}

impl From<Message> for OllamaMessage {
    fn from(message: Message) -> Self {
        OllamaMessage {
            role: message.role.to_string(),
            content: message.content,
        }
    }
}

impl From<&Message> for OllamaMessage {
    fn from(message: &Message) -> Self {
        OllamaMessage {
            role: message.role.to_string(),
            content: message.content.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Role;

    use super::*;
    #[ignore]
    #[tokio::test]
    async fn test_ollama_complete() {
        let adapter = OllamaAdapter::default();
        let messages = vec![Message {
            role: Role::User,
            content: "你好".to_string(),
        }];
        let content = adapter.complete(&messages).await.unwrap();
        println!("{}", content);
    }
}
