use metadata_qa_text::NAME as METADATA_QA_TEXT_NAME;
use swiftide::indexing::transformers::metadata_qa_text;
use swiftide_pgvector::{VectorStore, ask_query};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let questions = vec![
        "What is SwiftIDE? Provide a clear, comprehensive summary in under 50 words.".into(),
        "How can I use SwiftIDE to connect with the Ethereum blockchain? Please provide a concise, comprehensive summary in less than 50 words.".into(),
    ];
    let vector_store = VectorStore::try_new("rag_table", METADATA_QA_TEXT_NAME);
    // let llm_client = integrations::ollama::Ollama::default()
    //     .with_default_prompt_model("llama3.2")
    //     .to_owned();
    ask_query(
        vector_store.llm_client.clone(),
        vector_store.embed.clone(),
        vector_store.vector_store.clone(),
        questions,
    )
    .await?
    .iter()
    .enumerate()
    .for_each(|(i, result)| {
        info!("*** Answer Q{} ***", i + 1);
        info!("{}", result);
        info!("===X===");
    });
    Ok(())
}
