use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};

use crate::{AiAdapter, AiSdk, Message};

pub struct OpenAIAdapter {
    host: String,
    api_key: String,
    model: String,
    client: Client,
}

impl Default for OpenAIAdapter {
    fn default() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let model = "gpt-4o".to_string();
        let host = "https://api.openai.com".to_string();
        Self::new(api_key, model, host)
    }
}
#[derive(Serialize)]
pub struct OpenAIChatCompletion {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    pub usage: OpenAIUsage,
    pub service_tier: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
    pub prompt_tokens_details: OpenAITokenDetails,
    pub completion_tokens_details: OpenAICompletionTokenDetails,
}

#[derive(Deserialize, Debug)]
pub struct OpenAITokenDetails {
    pub cached_tokens: i32,
    pub audio_tokens: i32,
}

#[derive(Deserialize, Debug)]
pub struct OpenAICompletionTokenDetails {
    pub reasoning_tokens: i32,
    pub audio_tokens: i32,
    pub accepted_prediction_tokens: i32,
    pub rejected_prediction_tokens: i32,
}

#[derive(Deserialize, Debug)]
pub struct OpenAIChoice {
    pub index: u32,
    pub logprobs: Option<String>,
    pub message: OpenAIMessage,
    pub finish_reason: String,
}

impl OpenAIAdapter {
    pub fn new(api_key: String, model: String, host: String) -> Self {
        Self {
            host,
            api_key,
            model,
            client: Client::new(),
        }
    }
}

impl From<OpenAIAdapter> for AiAdapter {
    fn from(adapter: OpenAIAdapter) -> Self {
        AiAdapter::OpenAI(adapter)
    }
}

impl AiSdk for OpenAIAdapter {
    async fn complete(&self, messages: &[Message]) -> anyhow::Result<String> {
        let url = format!("{}/v1/chat/completions", self.host);
        let request = OpenAIChatCompletion {
            model: self.model.clone(),
            messages: messages.iter().map(|m| m.into()).collect(),
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;

        let mut data: OpenAIChatCompletionResponse = response.json().await?;
        println!("data: {:?}", data);
        //判断response.choices是否为空
        let content = data
            .choices
            .pop()
            .ok_or(anyhow::anyhow!("No response from OpenAI"))?
            .message
            .content;
        Ok(content)
    }
}

impl From<Message> for OpenAIMessage {
    fn from(message: Message) -> Self {
        OpenAIMessage {
            role: message.role.to_string(),
            content: message.content.clone(),
        }
    }
}

impl From<&Message> for OpenAIMessage {
    fn from(message: &Message) -> Self {
        OpenAIMessage {
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
    async fn test_openai_complete() {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();

        let adapter = OpenAIAdapter::new(
            api_key,
            "gpt-4o".to_string(),
            "https://api.openai.com".to_string(),
        );
        let messages = vec![Message {
            role: Role::User,
            content: "Hello, world!".to_string(),
        }];
        let response = adapter.complete(&messages).await.unwrap();
        println!("{}", response);
    }
}
