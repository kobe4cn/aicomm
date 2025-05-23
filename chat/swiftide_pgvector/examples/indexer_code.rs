// use metadata_qa_text::NAME as METADATA_QA_TEXT_NAME;
use swiftide::{
    indexing::{
        self, EmbeddedField, LanguageModelWithBackOff,
        loaders::FileLoader,
        transformers::{
            ChunkCode, Embed, MetadataQACode,
            metadata_qa_code::{self},
        },
    },
    integrations::{self, pgvector::PgVector},
};

use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // let vector_store = VectorStore::try_new("code_table", METADATA_QA_CODE_NAME).await?;

    // let openai_client = integrations::openai::OpenAI::builder()
    //     .default_embed_model("text-embedding-3-small")
    //     .default_prompt_model("gpt-3.5-turbo")
    //     .build()?;

    // let openai_client = LanguageModelWithBackOff::new(openai_client, Default::default());
    // let open_ai_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "".to_string());
    // info!("open_ai_key: {}", open_ai_key);

    // info!("Dropping existing table and index");
    // let drop_table_sql = "DROP TABLE IF EXISTS code_table";
    // let drop_index_sql = "DROP INDEX IF EXISTS code_table_embedding_idx";

    // if let Ok(pool) = vector_store.vector_store.get_pool().await {
    //     sqlx::query(drop_table_sql).execute(pool).await?;
    //     sqlx::query(drop_index_sql).execute(pool).await?;
    // } else {
    //     return Err("Failed to get database connection pool".into());
    // }

    // // let chunk_size = 2048;
    // info!("Starting indexing pipeline");
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
    info!("Indexing pipeline completed successfully");

    Ok(())
}

// #[derive(Clone)]
// #[allow(unused)]
// struct PrintMetadata;

// #[async_trait::async_trait]
// impl swiftide::traits::Transformer for PrintMetadata {
//     async fn transform_node(
//         &self,
//         node: swiftide::indexing::Node,
//     ) -> Result<swiftide::indexing::Node, anyhow::Error> {
//         println!("Metadata: {:?}", node);
//         Ok(node)
//     }
// }

// impl swiftide::traits::WithIndexingDefaults for PrintMetadata {}

// #[derive(Clone)]
// #[allow(unused)]
// struct AddFilenameToMetadata;

// #[async_trait::async_trait]
// impl Transformer for AddFilenameToMetadata {
//     async fn transform_node(
//         &self,
//         mut node: indexing::Node,
//     ) -> Result<indexing::Node, anyhow::Error> {
//         if let Some(path) = node.metadata.get("path").cloned() {
//             if let Some(path_str) = path.as_str() {
//                 let filename = std::path::Path::new(path_str)
//                     .file_name()
//                     .and_then(|n| n.to_str())
//                     .unwrap_or(path_str)
//                     .to_string();
//                 node.metadata.insert("path".to_string(), filename);
//             } else {
//                 return Err(anyhow::anyhow!("Path is not a string"));
//             };
//             // 只取文件名部分
//         }
//         Ok(node)
//     }
// }
