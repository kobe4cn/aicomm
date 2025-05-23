#[tokio::main]
async fn main() {
    let key = std::env::var("OPENAI_API_KEY").unwrap();
    println!("key: {}", key);
    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(key)
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await;
    println!("{:?}", resp);
}
