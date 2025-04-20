use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::IntoParams;

use crate::{AppError, AppState};

use super::ChatFile;

use core_lib::Message;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CreateMessage {
    pub files: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, IntoParams, Deserialize, Serialize)]
pub struct ListMessages {
    pub last_id: Option<u64>,
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

        let pool = &self.pool;
        let message = sqlx::query_as(
            r#"
            INSERT INTO messages(chat_id,sender_id,content,files)
            VALUES($1,$2,$3,$4)
            RETURNING id,chat_id,sender_id,content,files,created_at
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(input.files)
        .fetch_one(pool)
        .await?;
        Ok(message)
    }

    pub async fn list_messages(
        &self,
        input: ListMessages,

        chat_id: u64,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as _);
        let pool = &self.pool;
        let messages = sqlx::query_as(
            r#"
            SELECT id,chat_id,sender_id,content,files,created_at
            FROM messages
            WHERE chat_id=$1 and id < $2
            ORDER BY created_at DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(input.page_size as i64)
        .fetch_all(pool)
        .await?;
        Ok(messages)
    }
}
