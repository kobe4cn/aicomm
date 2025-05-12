use std::collections::HashSet;

use core_lib::Message;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use swiftide_pgvector::{VectorStore, ask_query};
use tokio_stream::StreamExt;
use tracing::info;
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
pub async fn setup_pg_listener(db_url: &str, bots: HashSet<i64>) -> anyhow::Result<()> {
    let mut listener = PgListener::connect(db_url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    let vector_store = VectorStore::try_new("chat_message_created", "chat_message_created");

    // let pgvector = vector_store.vector_store.clone();
    // let pool = pgvector.get_pool().await?;
    //多线程共享DashMap
    // let users = Arc::clone(&state.users);
    let pgvector = vector_store.vector_store.clone();
    tokio::spawn(async move {
        let pool = pgvector.get_pool().await.expect("get pool error");
        while let Some(Ok(notification)) = stream.next().await {
            info!("notification: {:?}", notification);
            if let Some(notification) =
                Notification::load(notification.channel(), notification.payload(), &bots)
            {
                let query = vec![notification.event.content.clone()];

                let result = ask_query(
                    vector_store.llm_client.clone(),
                    vector_store.embed.clone(),
                    vector_store.vector_store.clone(),
                    query,
                )
                .await;

                match result {
                    Ok(result) => {
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
                        .bind(result)
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
    });

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
}
