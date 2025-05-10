use core_lib::{AdapterType, AgentType, ChatAgents};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;

use crate::{AppError, AppState};

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CreateAgent {
    pub name: String,
    pub r#type: AgentType,
    pub prompt: String,
    pub args: serde_json::Value,
    pub adapter: AdapterType,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct UpdateAgent {
    pub id: u64,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub args: serde_json::Value,
    pub adapter: AdapterType,
    pub model: String,
}
impl CreateAgent {
    pub fn new(
        name: String,
        r#type: AgentType,
        prompt: String,
        args: serde_json::Value,
        adapter: AdapterType,
        model: String,
    ) -> Self {
        Self {
            name,
            r#type,
            prompt,
            args,
            adapter,
            model,
        }
    }
}

impl UpdateAgent {
    pub fn new(
        id: u64,
        prompt: String,
        args: serde_json::Value,
        adapter: AdapterType,
        model: String,
    ) -> Self {
        Self {
            id,
            prompt,
            args,
            adapter,
            model,
        }
    }
}

impl AppState {
    pub async fn create_agent(
        &self,
        input: CreateAgent,
        chat_id: u64,
        user_id: u64,
    ) -> Result<ChatAgents, AppError> {
        //user_id 必须存在于chat members里面
        let pool = &self.pool;
        let is_member = self.is_chat_member(chat_id as i64, user_id as i64).await?;
        if !is_member {
            info!("user_id {} not in chat_id {}", user_id, chat_id);
            return Err(AppError::UserNotInChat(format!(
                "user_id: {} not in chat_id: {}",
                user_id, chat_id
            )));
        }
        //check agents name exist
        if self.agent_name_exists(input.name.as_str(), chat_id).await? {
            info!(
                "agent name: {} already exists in chat_id: {}",
                input.name, chat_id
            );
            return Err(AppError::AgentExists(format!(
                "agent name: {} already exists in chat_id: {}",
                input.name, chat_id
            )));
        }
        let agent = sqlx::query_as(
            r#"
            INSERT INTO chat_agents (chat_id, name, type, prompt, args, adapter, model)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(chat_id as i64)
        .bind(input.name)
        .bind(input.r#type)
        .bind(input.prompt)
        .bind(input.args)
        .bind(input.adapter)
        .bind(input.model)
        .fetch_one(pool)
        .await?;
        Ok(agent)
    }

    pub async fn list_agents(
        &self,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Vec<ChatAgents>, AppError> {
        //user_id 必须存在于chat members里面
        let is_member = self.is_chat_member(chat_id as i64, user_id as i64).await?;
        if !is_member {
            info!("user_id {} not in chat_id {}", user_id, chat_id);
            return Err(AppError::UserNotInChat(format!(
                "user_id: {} not in chat_id: {}",
                user_id, chat_id
            )));
        }
        let pool = &self.pool;
        let agents = sqlx::query_as(
            r#"
            SELECT * FROM chat_agents WHERE chat_id = $1 order by id ASC
            "#,
        )
        .bind(chat_id as i64)
        .fetch_all(pool)
        .await?;
        Ok(agents)
    }

    //update an agent in a chat
    pub async fn update_agent(
        &self,
        input: UpdateAgent,
        chat_id: u64,
        user_id: u64,
    ) -> Result<ChatAgents, AppError> {
        let pool = &self.pool;
        let is_member = self.is_chat_member(chat_id as i64, user_id as i64).await?;
        if !is_member {
            info!("user_id {} not in chat_id {}", user_id, chat_id);
            return Err(AppError::UserNotInChat(format!(
                "user_id {} not in chat_id {}",
                user_id, chat_id
            )));
        }

        if !self.agent_id_exists(input.id, chat_id).await? {
            info!("agent id {} not found in chat_id {}", input.id, chat_id);
            return Err(AppError::AgentExists(format!(
                "agent id: {} not found in chat_id {}",
                input.id, chat_id
            )));
        }
        let prompt = input.prompt;
        let args = input.args;
        match (prompt.as_str(), &args) {
            ("", _) => {
                let agent = sqlx::query_as(
                    r#"
            UPDATE chat_agents SET args = $1, updated_at = now() WHERE chat_id = $2 and id = $3
            RETURNING *
            "#,
                )
                .bind(args)
                .bind(chat_id as i64)
                .bind(input.id as i64)
                .fetch_one(pool)
                .await?;
                Ok(agent)
            }
            (_, _) => {
                let agent = sqlx::query_as(
                    r#"
            UPDATE chat_agents SET prompt = $1, args = $2, updated_at = now() WHERE chat_id = $3 and id = $4
            RETURNING *
            "#,
                )
                .bind(prompt)
                .bind(args)
                .bind(chat_id as i64)
                .bind(input.id as i64)
                .fetch_one(pool)
                .await?;
                Ok(agent)
            }
        }
    }

    pub async fn agent_name_exists(&self, name: &str, chat_id: u64) -> Result<bool, AppError> {
        let pool = &self.pool;
        let exist = sqlx::query_scalar(
            r#"
            SELECT EXISTS(SELECT 1 FROM chat_agents WHERE name = $1 and chat_id = $2)
            "#,
        )
        .bind(name)
        .bind(chat_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(exist)
    }
    //check agent id exist
    pub async fn agent_id_exists(&self, id: u64, chat_id: u64) -> Result<bool, AppError> {
        let pool = &self.pool;
        let exist = sqlx::query_scalar(
            r#"
            SELECT EXISTS(SELECT 1 FROM chat_agents WHERE id = $1 and chat_id = $2)
            "#,
        )
        .bind(id as i64)
        .bind(chat_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(exist)
    }

    //get agent by id
    pub async fn get_agent_by_id(&self, id: u64) -> Result<Option<ChatAgents>, AppError> {
        let pool = &self.pool;
        let agent = sqlx::query_as(
            r#"
            SELECT * FROM chat_agents WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(agent)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::AppState;
    use anyhow::Result;

    #[tokio::test]
    async fn test_create_agent() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = CreateAgent {
            name: "test agent".to_string(),
            r#type: AgentType::Proxy,
            prompt: "you are a helpful assistant".to_string(),
            args: serde_json::Value::Null,
            adapter: AdapterType::Ollama,
            model: "llama3.2".to_string(),
        };
        // let chat = state.get_chat_by_id(2).await?;
        // println!("chat: {:?}", chat);
        let agent = state.create_agent(input, 2, 1).await?;
        assert_eq!(agent.name, "test agent");
        assert_eq!(agent.r#type, AgentType::Proxy);
        assert_eq!(agent.prompt, "you are a helpful assistant");
        assert_eq!(agent.adapter, AdapterType::Ollama);
        assert_eq!(agent.model, "llama3.2");
        // assert_eq!(agent.args, serde_json::Value::Null);
        Ok(())
    }

    #[tokio::test]
    async fn test_update_agent() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let input = UpdateAgent {
            id: 2,
            prompt: "you are a helpful assistant".to_string(),
            args: serde_json::json!({
                "model": "llama3.2"
            }),
            adapter: AdapterType::Ollama,
            model: "llama3.2".to_string(),
        };
        let agent = state.update_agent(input, 2, 1).await?;
        assert_eq!(agent.prompt, "you are a helpful assistant");
        assert_eq!(agent.args["model"], "llama3.2");
        Ok(())
    }

    #[tokio::test]
    async fn test_list_agents() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let agents = state.list_agents(2, 1).await?;
        assert_eq!(agents.len(), 2);
        Ok(())
    }
}
