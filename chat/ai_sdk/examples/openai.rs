use ai_sdk::{AiSdk, Message, OpenAIAdapter, Role};

#[tokio::main]
async fn main() {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let adapter = OpenAIAdapter::new(
        api_key,
        "gpt-4o".to_string(),
        "https://api.openai.com".to_string(),
    );
    let messages = vec![Message {
        role: Role::User,
        content: "Hello, world!".to_string(),
    }];
    let response = adapter.complete(&messages).await.unwrap();
    println!("{}", response);
}
