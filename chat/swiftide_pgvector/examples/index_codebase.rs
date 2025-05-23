//! # [Swiftide] Indexing the Swiftide itself example
//!
//! This example demonstrates how to index the Swiftide codebase itself.
//! Note that for it to work correctly you need to have OPENAI_API_KEY set, redis and qdrant
//! running.
//!
//! The pipeline will:
//! - Load all `.rs` files from the current directory
//! - Skip any nodes previously processed; hashes are based on the path and chunk (not the
//!   metadata!)
//! - Run metadata QA on each chunk; generating questions and answers and adding metadata
//! - Chunk the code into pieces of 10 to 2048 bytes
//! - Embed the chunks in batches of 10, Metadata is embedded by default
//! - Store the nodes in Qdrant
//!
//! Note that metadata is copied over to smaller chunks when chunking. When making LLM requests
//! with lots of small chunks, consider the rate limits of the API.
//!
//! [Swiftide]: https://github.com/bosun-ai/swiftide
//! [examples]: https://github.com/bosun-ai/swiftide/blob/master/examples

use swiftide::{
    indexing::{
        self, EmbeddedField, LanguageModelWithBackOff,
        loaders::FileLoader,
        transformers::{ChunkCode, Embed, MetadataQACode, metadata_qa_code},
    },
    integrations::{self, pgvector::PgVector},
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let openai_client = integrations::openai::OpenAI::builder()
        .default_embed_model("text-embedding-3-small")
        .default_prompt_model("gpt-3.5-turbo")
        .build()?;

    let openai_client = LanguageModelWithBackOff::new(openai_client, Default::default());

    let pgv_storage = PgVector::builder()
        .db_url("postgresql://postgres:postgres@localhost:5432/chat")
        .vector_size(1536)
        .with_vector(EmbeddedField::Combined)
        .with_metadata(metadata_qa_code::NAME)
        .table_name("code_table")
        .build()
        .unwrap();
    info!("Dropping existing table and index");
    let drop_table_sql = "DROP TABLE IF EXISTS code_table";
    let drop_index_sql = "DROP INDEX IF EXISTS code_table_embedding_idx";

    if let Ok(pool) = pgv_storage.get_pool().await {
        sqlx::query(drop_table_sql).execute(pool).await?;
        sqlx::query(drop_index_sql).execute(pool).await?;
    } else {
        return Err("Failed to get database connection pool".into());
    }

    indexing::Pipeline::from_loader(FileLoader::new("./examples").with_extensions(&["rs"]))
        // .filter_cached(Redis::try_from_url(redis_url, "swiftide-examples")?)
        .then(MetadataQACode::new(openai_client.clone()))
        .then_chunk(ChunkCode::try_for_language_and_chunk_size(
            "rust",
            10..2048,
        )?)
        .then_in_batch(Embed::new(openai_client.clone()).with_batch_size(10))
        .then_store_with(pgv_storage)
        .run()
        .await?;
    Ok(())
}
