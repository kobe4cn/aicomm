// use metadata_qa_text::NAME as METADATA_QA_TEXT_NAME;
use swiftide::{
    indexing::{
        self,
        loaders::FileLoader,
        transformers::{
            ChunkCode, Embed, MetadataQACode, metadata_qa_code::NAME as METADATA_QA_CODE_NAME,
        },
    },
    integrations::{self},
};
use swiftide_pgvector::VectorStore;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{Layer as _, fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let vector_store = VectorStore::try_new("code_table", METADATA_QA_CODE_NAME);

    let llm_client = integrations::ollama::Ollama::default()
        .with_default_prompt_model("llama3.2")
        .to_owned();

    // let pgv_storage = PgVector::builder()
    //     .db_url("postgresql://postgres:postgres@localhost:5432/swiftide_ray")
    //     .vector_size(384)
    //     .with_vector(EmbeddedField::Combined)
    //     .with_metadata(METADATA_QA_CODE_NAME)
    //     .table_name("code_table")
    //     .build()
    //     .unwrap();

    // let fastembed =
    //     integrations::fastembed::FastEmbed::try_default().expect("could not create fastembed");

    info!("Dropping existing table and index");
    let drop_table_sql = "DROP TABLE IF EXISTS code_table";
    let drop_index_sql = "DROP INDEX IF EXISTS code_table_embedding_idx";

    if let Ok(pool) = vector_store.vector_store.get_pool().await {
        sqlx::query(drop_table_sql).execute(pool).await?;
        sqlx::query(drop_index_sql).execute(pool).await?;
    } else {
        return Err("Failed to get database connection pool".into());
    }

    let chunk_size = 1024;
    info!("Starting indexing pipeline");

    indexing::Pipeline::from_loader(FileLoader::new("./examples/").with_extensions(&["rs"]))
        .then_chunk(ChunkCode::try_for_language_and_chunk_size(
            "rust", chunk_size,
        )?)
        .then(MetadataQACode::new(llm_client.clone()))
        .then_in_batch(Embed::new(vector_store.embed.clone()).with_batch_size(30))
        .then(indexing::transformers::CompressCodeOutline::new(
            llm_client.clone(),
        ))
        .then(PrintMetadata)
        .then_store_with(vector_store.vector_store.clone())
        .run()
        .await?;

    info!("Indexing pipeline completed successfully");

    Ok(())
}

#[derive(Clone)]
#[allow(unused)]
struct PrintMetadata;

#[async_trait::async_trait]
impl swiftide::traits::Transformer for PrintMetadata {
    async fn transform_node(
        &self,
        node: swiftide::indexing::Node,
    ) -> Result<swiftide::indexing::Node, anyhow::Error> {
        println!("Metadata: {:?}", node);
        Ok(node)
    }
}

impl swiftide::traits::WithIndexingDefaults for PrintMetadata {}

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
