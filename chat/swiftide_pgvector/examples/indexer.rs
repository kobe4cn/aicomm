use metadata_qa_text::NAME as METADATA_QA_TEXT_NAME;
use swiftide::{
    indexing::{
        self, LanguageModelWithBackOff,
        loaders::FileLoader,
        transformers::{ChunkMarkdown, Embed, MetadataQAText, metadata_qa_text},
    },
    integrations,
};
use swiftide_pgvector::VectorStore;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let vector_store =
        VectorStore::try_new("rag_table", METADATA_QA_TEXT_NAME, "localhost").await?;

    let llm_client = integrations::openai::OpenAI::builder()
        .default_embed_model("text-embedding-3-small")
        .default_prompt_model("gpt-4o")
        .build()?;
    let llm_client = LanguageModelWithBackOff::new(llm_client, Default::default());
    // let fastembed =
    //     integrations::fastembed::FastEmbed::try_default().expect("could not create fastembed");

    info!("Dropping existing table and index");
    let drop_table_sql = "DROP TABLE IF EXISTS rag_table";
    let drop_index_sql = "DROP INDEX IF EXISTS rag_table_embedding_idx";

    if let Ok(pool) = vector_store.vector_store.get_pool().await {
        sqlx::query(drop_table_sql).execute(pool).await?;
        sqlx::query(drop_index_sql).execute(pool).await?;
    } else {
        return Err("Failed to get database connection pool".into());
    }

    info!("Starting indexing pipeline");

    indexing::Pipeline::from_loader(FileLoader::new("README.md"))
        .then_chunk(ChunkMarkdown::from_chunk_range(10..2048))
        .then(MetadataQAText::new(llm_client.clone()))
        .then_in_batch(Embed::new(llm_client.clone()).with_batch_size(100))
        .then_store_with(vector_store.vector_store.clone())
        .run()
        .await?;

    info!("Indexing pipeline completed successfully");

    Ok(())
}
