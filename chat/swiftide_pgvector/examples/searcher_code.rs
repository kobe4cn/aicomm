use metadata_qa_code::NAME as METADATA_QA_CODE_NAME;
use swiftide::indexing::transformers::metadata_qa_code;
use swiftide_pgvector::{VectorStore, ask_query};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let questions = vec!["这个代码在做什么事情？请用中文简单回答".into()];
    let vector_store =
        VectorStore::try_new("code_table", METADATA_QA_CODE_NAME, "localhost").await?;

    ask_query(
        vector_store.llm_client.clone(),
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
