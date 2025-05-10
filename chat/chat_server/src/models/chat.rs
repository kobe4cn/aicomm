use crate::{AppError, AppState};
use core_lib::{Chat, ChatType};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, ToSchema, Deserialize, Serialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl AppState {
    pub async fn create_chat(&self, input: CreateChat, ws_id: u64) -> Result<Chat, AppError> {
        let pool = &self.pool;
        //对话成员必须大于2人
        let chat_type = self.verify_chat_type(&input).await?;

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats(ws_id,name,type,members)
            VALUES($1,$2,$3,$4)
            RETURNING id,ws_id,name,type,members,created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }

    pub async fn fetch_all_chat(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let pool = &self.pool;
        let chats = sqlx::query_as(
            r#"
            SELECT id,ws_id,name,type,members,created_at
            FROM chats
            WHERE ws_id=$1 order by created_at desc
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;
        Ok(chats)
    }

    pub async fn get_chat_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id,ws_id,name,type,members,created_at
            FROM chats
            WHERE id=$1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat)
    }
    pub async fn update_chat(&self, input: CreateChat, id: u64) -> Result<Chat, AppError> {
        let chat_type = self.verify_chat_type(&input).await?;

        let chat = sqlx::query_as(
            r#"
            UPDATE chats
            SET name=$1,type=$2,members=$3
            WHERE id=$4
            RETURNING id,ws_id,name,type,members,created_at
            "#,
        )
        .bind(input.name)
        .bind(chat_type)
        .bind(input.members)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }
    pub async fn delete_chat(&self, id: u64) -> Result<Chat, AppError> {
        let chat = sqlx::query_as(
            r#"
            DELETE FROM chats
            WHERE id=$1
            RETURNING id,ws_id,name,type,members,created_at
            "#,
        )
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn verify_chat_type(&self, input: &CreateChat) -> Result<ChatType, AppError> {
        //对话成员必须大于2人
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }

        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }

        //verify if all members exist
        let users = self.fetch_chat_user_by_ids(&input.members).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };
        Ok(chat_type)
    }

    pub async fn is_chat_member(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
        let chat = sqlx::query(
            r#"
            SELECT id
            FROM chats
            WHERE id=$1 and $2=ANY(members)
            "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}
#[cfg(test)]
mod tests {
    use anyhow::Result;

    use anyhow::Ok;
    use core_lib::ChatType;

    use crate::models::CreateChat;
    use crate::AppState;

    #[tokio::test]
    async fn test_create_chat() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateChat::new("", &[1, 2], false);
        let ws_id = 1;
        let chat = state
            .create_chat(input, ws_id)
            .await
            .expect("create chat failed");
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_group_chat() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let ws_id = 1;
        let members = &[1, 2, 3];
        let input = CreateChat::new("", members, false);
        let chat = state.create_chat(input, ws_id).await.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.r#type, ChatType::Group);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_public_chat_with_name() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let ws_id = 1;
        let members = &[1, 2, 3];
        let input = CreateChat::new("public chat", members, true);
        let chat = state.create_chat(input, ws_id).await.unwrap();
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }
}
