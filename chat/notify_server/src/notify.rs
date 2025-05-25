use std::{collections::HashSet, sync::Arc};

use core_lib::{Chat, Message};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgListener;
use tracing::info;

use crate::AppState;
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum AppEvent {
    NewChat(Chat),
    NewMessage(Message),
    AddToChat(Chat),
    UpdateChatName(Chat),
    RemoveFromChat(Chat),
}
#[derive(Debug)]
struct Notification {
    //user being impact
    user_ids: HashSet<u64>,
    event: Arc<AppEvent>,
}

//通过serde json需要转换的收到数据的tragger的事件数据结构
/*'chat_updated',
json_build_object(
    'op',
    TG_OP,
    'old',
    OLD,
    'new',
    NEW
)*/
#[derive(Debug, Serialize, Deserialize)]
struct ChatUpdated {
    op: String,
    old: Option<Chat>,
    new: Option<Chat>,
}
//'chat_message_created',row_to_json(NEW)
#[derive(Debug, Serialize, Deserialize)]
struct ChatMessageCreated {
    members: Vec<i64>,

    message: Message,
}

pub async fn setup_pg_listener(state: AppState) -> anyhow::Result<()> {
    let db_url = &state.config.server.db_url;
    let mut listener = PgListener::connect(db_url.as_str()).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;
    let mut stream = listener.into_stream();

    //多线程共享DashMap
    let users = Arc::clone(&state.users);

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(notification) => {
                    let notification =
                        match Notification::load(notification.channel(), notification.payload()) {
                            Ok(n) => n,
                            Err(e) => {
                                info!("Failed to load notification: {:?}", e);
                                continue;
                            }
                        };

                    //如果在tx.send中remove 已经发送失败的用户，会导致其他影响失效。对于.is_err()的用户，需要在发送失败后再移除
                    //将失败的用户保存进入failed_users vec
                    let mut failed_users = Vec::new();
                    for user_id in notification.user_ids {
                        if let Some(tx) = users.get(&user_id) {
                            info!(
                                "notification: {:?} to user {}",
                                notification.event.clone(),
                                &user_id
                            );
                            if tx.send(notification.event.clone()).is_err() {
                                info!("send event failed for user {}", user_id);
                                failed_users.push(user_id);
                                // 移除用户
                                info!("user need move from the map: {:?}", failed_users);
                            }
                        }
                    }
                    //遍历failed_users vec 从dashmap删除用户
                    for user_id in failed_users {
                        if users.remove(&user_id).is_some() {
                            info!("user {} removed successfully.", user_id);
                        }
                    }

                    info!("send event success");
                }
                Err(e) => {
                    info!("stream error: {:?}", e);
                    continue;
                    // 可视情况决定继续或终止
                }
            }
        }
        Ok::<(), anyhow::Error>(())
    });

    Ok(())
}

impl Notification {
    fn load(r#type: &str, playload: &str) -> anyhow::Result<Self> {
        match r#type {
            "chat_updated" => {
                let payload: ChatUpdated = serde_json::from_str(playload)?;
                let user_ids = get_affected_user_ids(payload.old.as_ref(), payload.new.as_ref());
                let event = match payload.op.as_str() {
                    "INSERT" => AppEvent::NewChat(payload.new.expect("new should exist")),
                    "UPDATE" => {
                        if check_chat_name_update(payload.old.as_ref(), payload.new.as_ref()) {
                            AppEvent::UpdateChatName(payload.new.expect("new should exist"))
                        } else {
                            AppEvent::AddToChat(payload.new.expect("new should exist"))
                        }
                    }
                    "DELETE" => AppEvent::RemoveFromChat(payload.old.expect("old should exist")),
                    _ => return Err(anyhow::anyhow!("Invalid op")),
                };
                info!("user_ids: {:?}, event :{:?}", user_ids, event);
                Ok(Self {
                    user_ids,
                    event: Arc::new(event),
                })
            }
            "chat_message_created" => {
                let payload: ChatMessageCreated = serde_json::from_str(playload)?;
                let user_ids = payload.members.iter().map(|v| *v as u64).collect();
                Ok(Self {
                    user_ids,
                    event: Arc::new(AppEvent::NewMessage(payload.message)),
                })
            }
            _ => Err(anyhow::anyhow!("Invalid type")),
        }
    }
}

fn get_affected_user_ids(old: Option<&Chat>, new: Option<&Chat>) -> HashSet<u64> {
    match (old, new) {
        (Some(old), Some(new)) => {
            let old_user_ids: HashSet<_> = old.members.iter().map(|v| *v as u64).collect();
            let new_user_ids: HashSet<_> = new.members.iter().map(|v| *v as u64).collect();
            if old_user_ids == new_user_ids {
                new_user_ids
            } else {
                old_user_ids.union(&new_user_ids).copied().collect()
            }
        }
        (Some(old), None) => old.members.iter().map(|v| *v as u64).collect(),
        (None, Some(new)) => new.members.iter().map(|v| *v as u64).collect(),
        (None, None) => HashSet::new(),
    }
}
fn check_chat_name_update(old: Option<&Chat>, new: Option<&Chat>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => {
            old.name != new.name && old.members == new.members && old.r#type == new.r#type
        }
        _ => false,
    }
}
