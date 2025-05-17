use std::net::SocketAddr;

use anyhow::Result;

use chat_server::AppState;
use core_lib::{Chat, ChatType, Message};
use futures::StreamExt;
use notify_server::AppConfig;
use reqwest::{
    multipart::{Form, Part},
    StatusCode,
};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;

#[derive(Debug)]
struct ChatServer {
    // server fields here
    addr: SocketAddr,
    token: String,
    client: reqwest::Client,
}
// struct NotifyServer;

#[derive(Debug, Deserialize)]
struct AuthToken {
    token: String,
}
const WILD_ADDR: &str = "0.0.0.0:0";
// #[tokio::test]
// async fn chat_server_should_work() -> Result<()> {
//     let (tdb, state) = chat_server::AppState::new_for_test().await?;
//     let chat_server = ChatServer::new(state).await?;
//     let mut config = AppConfig::try_load()?;
//     config.server.db_url = tdb.url().to_string();
//     NotifyServer::new(&chat_server.token, config).await?;
//     let chat = chat_server.create_chat().await?;
//     let _msg = chat_server.create_message(chat).await?;
//     sleep(Duration::from_secs(1)).await;
//     // test code here
//     Ok(())
// }
#[allow(unused)]
impl ChatServer {
    async fn new(state: AppState) -> Result<Self> {
        // server initialization code here
        let app = chat_server::get_router(state).await?;

        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut ret = Self {
            addr,
            token: "".to_string(),
            client: reqwest::Client::new(),
        };
        ret.token = ret.signin().await?;
        Ok(ret)
    }

    async fn signin(&self) -> Result<String> {
        let res = self
            .client
            .post(format!("http://{}/api/signin", self.addr))
            .header("content-type", "application/json")
            .body(r#"{"email":"kevin.yang.xgz@gmail.com","password":"test123456"}"#)
            .send()
            .await?;
        assert_eq!(res.status(), 200);
        let ret = res.json::<AuthToken>().await?;

        Ok(ret.token)
    }

    async fn create_chat(&self) -> Result<Chat> {
        let res = self
            .client
            .post(format!("http://{}/api/chats", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(
                r#"{
  "name": "test chat",
  "members": [1,2,3],
  "public": false
}"#,
            )
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::CREATED);
        let ret = res.json::<Chat>().await?;
        assert_eq!(ret.name.as_ref().unwrap(), "test chat");

        Ok(ret)
    }

    async fn create_message(&self, chat: Chat) -> Result<Message> {
        let file_data = include_bytes!("../Cargo.toml");

        let files = Part::bytes(file_data)
            .file_name("Cargo.toml")
            .mime_str("text/plain")?;
        let form = Form::new().part("file", files);
        let res: reqwest::Response = self
            .client
            .post(format!("http://{}/api/upload", self.addr))
            .header("Authorization", format!("Bearer {}", self.token))
            .multipart(form)
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::OK);
        let ret: Vec<String> = res.json().await?;

        let body = serde_json::to_string(&json!({
          "content": "hello,lei yang",
          "files": ret,
        }))?;

        let res = self
            .client
            .post(format!("http://{}/api/chats/{}", self.addr, chat.id))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;
        // assert_eq!(res.status(), StatusCode::CREATED);
        let ret = res.json::<Message>().await?;
        // assert_eq!(ret.content, "hello,lei yang");
        Ok(ret)
    }
}

impl NotifyServer {
    async fn new(token: &str, config: AppConfig) -> Result<Self> {
        // server initialization code here
        let app = notify_server::get_router(config).await?;

        let listener = TcpListener::bind(WILD_ADDR).await?;
        let addr = listener.local_addr()?;
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        });

        let mut es = EventSource::get(format!("http://{}/events?access_token={}", addr, token));
        tokio::spawn(async move {
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => println!("Connection Open!"),

                    Ok(Event::Message(message)) => match message.event.as_str() {
                        "NewChat" => {
                            let chat: Chat = serde_json::from_str(&message.data).unwrap();
                            println!("message {}", message.data);
                            assert_eq!(chat.name.as_ref().unwrap(), "test chat");
                            assert_eq!(chat.members, vec![1, 2, 3]);
                            assert_eq!(chat.r#type, ChatType::PrivateChannel);
                        }
                        "NewMessage" => {
                            let msg: Message = serde_json::from_str(&message.data).unwrap();
                            println!("message {}", message.data);
                            assert_eq!(msg.content, "hello,lei yang");
                            assert_eq!(msg.files.len(), 1);
                        }
                        _ => {
                            println!("Unknown event: {:?}", message);
                        }
                    },
                    Err(err) => {
                        println!("Error: {}", err);
                        es.close();
                    }
                }
            }
        });

        Ok(Self)
    }
}
