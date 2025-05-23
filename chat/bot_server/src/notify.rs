use core_lib::Message;
use metadata_qa_code::NAME as METADATA_QA_CODE_NAME;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgListener};
use std::collections::HashSet;
use swiftide::indexing::transformers::metadata_qa_code;
use swiftide_pgvector::{VectorStore, ask_query};
use tokio_stream::StreamExt;
use tracing::info;

use crate::config::AppConfig;
#[derive(Debug)]
#[allow(unused)]
struct Notification {
    //user being impact
    user_id: i64,
    event: Message,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    members: HashSet<i64>,
    message: Message,
}

#[allow(unused)]
pub async fn setup_pg_listener(config: &AppConfig) -> anyhow::Result<()> {
    let mut listener = PgListener::connect(&config.server.db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    let vector_store = VectorStore::try_new("code_table", METADATA_QA_CODE_NAME)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create vector store: {}", e))?;

    // let pgvector = vector_store.vector_store.clone();
    // let pool = pgvector.get_pool().await?;
    //多线程共享DashMap
    // let users = Arc::clone(&state.users);
    let pgvector = vector_store.vector_store.clone();

    let pool = pgvector.get_pool().await.expect("get pool error");
    let bots = Notification::get_bots(pool).await.expect("get bots error");
    while let Some(Ok(notification)) = stream.next().await {
        if let Some(notification) =
            Notification::load(notification.channel(), notification.payload(), &bots)
        {
            let query = vec![notification.event.content.clone()];
            info!("process notification: {:?}", query);
            let result = ask_query(
                vector_store.llm_client.clone(),
                vector_store.vector_store.clone(),
                query,
            )
            .await;

            match result {
                Ok(result) => {
                    info!("Got an answer: {:?}", result.first());
                    // let reply = result.join(" ");
                    // info!("reply: {}", reply);
                    let _id: i64 = sqlx::query_scalar(
                        r#"
                            INSERT INTO messages(chat_id,sender_id,content)
                            VALUES($1,$2,$3)
                            RETURNING id
                            "#,
                    )
                    .bind(notification.event.chat_id)
                    .bind(notification.user_id)
                    .bind(result.first())
                    .fetch_one(pool)
                    .await
                    .expect("send message error");
                }

                Err(e) => {
                    info!("reply error: {:?}", e);
                }
            }
        }
    }

    Ok(())
}

#[allow(unused)]
impl Notification {
    fn load(r#type: &str, payload: &str, bots: &HashSet<i64>) -> Option<Self> {
        match r#type {
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(payload).ok()?;
                let mut members = payload.members;
                members.remove(&payload.message.sender_id);

                if members.len() == 1 {
                    let bot_id = members.iter().next().unwrap();
                    if bots.contains(bot_id) {
                        return Some(Self {
                            user_id: *bot_id,
                            event: payload.message,
                        });
                    }
                }
                None
            }
            _ => None,
        }
    }

    async fn get_bots(pool: &PgPool) -> anyhow::Result<HashSet<i64>> {
        let bots: Vec<(i64,)> = sqlx::query_as(r#"SELECT id FROM users WHERE is_bot = true"#)
            .fetch_all(pool)
            .await?;
        Ok(bots.into_iter().map(|(id)| id.0).collect())
    }
}
