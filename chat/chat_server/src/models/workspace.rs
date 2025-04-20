use crate::{AppError, AppState};

use core_lib::{ChatUser, WorkSpace};

impl AppState {
    pub async fn create_workspace(&self, name: &str, user_id: u64) -> Result<WorkSpace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            INSERT INTO workspaces(name,owner_id)
            VALUES($1,$2)
            RETURNING id,name,owner_id,created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(workspace)
    }

    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<WorkSpace>, AppError> {
        let workspace = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE name=$1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(workspace)
    }
    #[allow(unused)]
    pub async fn find_workspace_by_id(&self, id: u64) -> Result<Option<WorkSpace>, AppError> {
        let workspace = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspaces
            WHERE id=$1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(workspace)
    }

    pub async fn fetch_all_chat_users(&self, workspace_id: u64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id,fullname,email
            FROM users
            WHERE ws_id = $1 order by id
            "#,
        )
        .bind(workspace_id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
    pub async fn update_workspace_owner(
        &self,
        id: u64,
        new_owner_id: u64,
    ) -> Result<WorkSpace, AppError> {
        let workspace = sqlx::query_as(
            r#"
            UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2 and (select ws_id from users where id=$1)=$2
            RETURNING id,name,owner_id,created_at
            "#,
        )
        .bind(new_owner_id as i64)
        .bind(id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(workspace)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use anyhow::Result;

    #[tokio::test]
    async fn test_create_workspace() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await?;
        assert_eq!(ws.name, "test");
        Ok(())
    }
}
