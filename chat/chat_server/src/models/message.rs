use std::str::FromStr;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use utoipa::IntoParams;

use crate::{agent::AgentVariant, AppError, AppState};

use super::ChatFile;

use core_lib::{Agent, AgentContext, AgentDecision, ChatType, Message};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CreateMessage {
    pub files: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, IntoParams, Deserialize, Serialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
    #[serde(default)]
    pub page_size: u64,
}

impl AppState {
    #[allow(unused)]
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        if input.content.is_empty() {
            return Err(AppError::MessageCreateError(
                "content is required".to_string(),
            ));
        }
        for s in &input.files {
            let base_dir = &self.config.server.base_dir;
            let file = ChatFile::from_str(s)?;

            if !file.path(base_dir).exists() {
                return Err(AppError::MessageCreateError("file not exists".to_string()));
            }
        }
        //check chat_id exists and user_id in this chat
        let chat = self.is_chat_member(chat_id as i64, user_id as i64).await?;

        if !chat {
            return Err(AppError::MessageCreateError(
                "chat not exists or user not in this chat".to_string(),
            ));
        }
        //get the agent and process the message
        let mut agents = self.list_agents(chat_id, user_id).await?;
        info!("agents: {:?}", agents);
        let decision = if let Some(agent) = agents.pop() {
            let agent: AgentVariant = agent.into();
            agent
                .process(
                    &input.content,
                    &AgentContext::new(chat_id as i64, user_id as i64, 1),
                )
                .await?
        } else {
            AgentDecision::None
        };

        let modified_content = match &decision {
            AgentDecision::Modify(content) => Some(content.clone()),
            _ => None,
        };

        let pool = &self.pool;
        let message = sqlx::query_as(
            r#"
            INSERT INTO messages(chat_id,sender_id,content,files,modified_content)
            VALUES($1,$2,$3,$4,$5)
            RETURNING *
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(input.files)
        .bind(modified_content)
        .fetch_one(pool)
        .await?;

        //if decision is reply, create a new message
        if let AgentDecision::Reply(reply) = decision {
            let chat = self
                .get_chat_by_id(chat_id as i64)
                .await?
                .expect("chat not found");
            if chat.r#type != ChatType::Single {
                warn!(
                    "reply decision found in non single chat {}. reply:{:?}",
                    chat_id, reply
                );
            }
            // let other_user = chat
            //     .members
            //     .iter()
            //     .find(|&&id| id != user_id as i64)
            //     .expect("other user not found");

            // let _id: i64 = sqlx::query_scalar(
            //     r#"
            //     INSERT INTO messages(chat_id,sender_id,content)
            //     VALUES($1,$2,$3)
            //     RETURNING id
            //     "#,
            // )
            // .bind(chat_id as i64)
            // .bind(other_user)
            // .bind(reply)
            // .fetch_one(pool)
            // .await?;
        }

        Ok(message)
    }

    pub async fn list_messages(
        &self,
        input: ListMessages,

        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);
        let page_size = input.page_size.clamp(100, 500) as i64;
        let pool = &self.pool;

        println!("last_id: {} {} {}", last_id, chat_id, page_size);
        let messages = sqlx::query_as(
            r#"
            SELECT id,chat_id,sender_id,content,files,modified_content,created_at
            FROM messages
            WHERE chat_id=$1 and id < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(page_size)
        .fetch_all(pool)
        .await?;
        Ok(messages)
    }
}
