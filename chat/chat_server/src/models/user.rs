use std::mem;

use crate::{AppError, AppState};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};

use core_lib::{ChatUser, User};
use tracing::info;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema, Serialize)]
pub struct CreateUser {
    pub fullname: String,
    pub email: String,
    pub password: String,
    pub workspace: String,
}
#[derive(Debug, Clone, Deserialize, ToSchema, Serialize)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

impl AppState {
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            r#"SELECT id,ws_id,fullname,email,created_at,is_bot FROM users WHERE email=$1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    // TODO: transaction thinking for workspace create rollback
    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        //check if workspaces
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password_hash = hash_password(&input.password)?;

        //check if email is bot TODO: move to a function
        let is_bot = input.email.ends_with("@bot.com");
        let user: User = sqlx::query_as(
            r#"INSERT INTO users (ws_id,fullname,email,password_hash,is_bot) VALUES ($1,$2,$3,$4,$5) RETURNING id,ws_id,fullname,email,created_at,is_bot"#,
        )
        .bind(ws.id)
        .bind(&input.fullname)
        .bind(&input.email)
        .bind(password_hash)
        .bind(is_bot)
        .fetch_one(&self.pool)
        .await?;
        self.update_workspace_owner(ws.id as _, user.id as _)
            .await?;
        Ok(user)
    }
    #[allow(unused)]
    pub async fn add_user_to_workspace(
        &self,
        workspace_id: u64,
        user_id: u64,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as(
            r#"UPDATE users SET ws_id=$1 WHERE id=$2
        RETURNING id,ws_id,fullname,email,created_at
        "#,
        )
        .bind(workspace_id as i64)
        .bind(user_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }
    ///verify email and password
    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        info!("input: {:?}", input);

        let user: Option<User> = sqlx::query_as(
            r#"SELECT id,ws_id,fullname,email,password_hash,created_at,is_bot FROM users WHERE email=$1"#,
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;
        info!("user: {:?}", user);
        match user {
            Some(mut user) => {
                let password_hash = mem::take(&mut user.password_hash);
                let is_valid =
                    verify_password(&input.password, &password_hash.unwrap_or_default())?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
    pub async fn fetch_chat_user_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(r#"SELECT id,fullname,email FROM users WHERE id=ANY($1)"#)
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;
        Ok(users)
    }

    // pub async fn fetch_all_chat_users(&self, ws_id: u64) -> Result<Vec<ChatUser>, AppError> {
    //     let users = sqlx::query_as(r#"SELECT id,fullname,email FROM users WHERE ws_id=$1"#)
    //         .bind(ws_id as i64)
    //         .fetch_all(&self.pool)
    //         .await?;
    //     Ok(users)
    // }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            workspace: "default".to_string(),
            fullname: fullname.to_string(),
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[allow(unused)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;
    #[tokio::test]

    async fn create_and_verify_user_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let email = "kevin.yang.xgz1@gamil.com";
        let input = CreateUser::new("kevin yang", email, "password123456");
        let user = state.create_user(&input).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, "kevin yang");

        let user = state.find_user_by_email(email).await?;
        assert!(user.is_some());
        assert_eq!(user.unwrap().email, email);
        let signinuser = SigninUser::new(email, "password123456");
        let user = state.verify_user(&signinuser).await?;
        assert!(user.is_some());
        assert!(user.unwrap().email == email);
        // do something with the pool

        // when tdb gets dropped, the database will be dropped
        Ok(())
    }
}
