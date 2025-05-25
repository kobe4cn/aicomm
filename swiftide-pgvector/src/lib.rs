use swiftide::{
    indexing::EmbeddedField,
    integrations::{self, fastembed::FastEmbed, pgvector::PgVector},
    query::{self, answers, query_transformers, response_transformers},
    traits::SimplePrompt,
};


#[allow(unused)]
pub struct VectorStore {
    pub vector_store: PgVector,
    pub embed: FastEmbed,
    pub llm_client: Box<dyn SimplePrompt>,
}

impl VectorStore {
    pub fn try_new(table_name: &str, metadata: &str) -> Self {
        let pgv_storage = PgVector::builder()
            .db_url("postgresql://postgres:postgres@localhost:5432/chat")
            .vector_size(384)
            .with_vector(EmbeddedField::Combined)
            .with_metadata(metadata)
            .table_name(table_name)
            .build()
            .unwrap();
        let llm_client = integrations::ollama::Ollama::default()
            .with_default_prompt_model("llama3.2")
            .to_owned();

        let fastembed =
            integrations::fastembed::FastEmbed::try_default().expect("could not create fastembed");

        Self::new(pgv_storage, fastembed, Box::new(llm_client))
    }
    pub fn new(
        vector_store: PgVector,
        embed: FastEmbed,
        llm_client: Box<dyn SimplePrompt>,
    ) -> Self {
        Self {
            vector_store,
            embed,
            llm_client,
        }
    }
}

pub async fn ask_query(
    llm_client: Box<dyn SimplePrompt>,
    embed: FastEmbed,
    vector_store: PgVector,
    questions: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // By default the search strategy is SimilaritySingleEmbedding
    // which takes the latest query, embeds it, and does a similarity search
    //
    // Pgvector will return an error if multiple embeddings are set
    //
    // The pipeline generates subquestions to increase semantic coverage, embeds these in a single
    // embedding, retrieves the default top_k documents, summarizes them and uses that as context
    // for the final answer.

    let pipeline = query::Pipeline::default()
        .then_transform_query(query_transformers::GenerateSubquestions::from_client(
            llm_client.clone(),
        ))
        .then_transform_query(query_transformers::Embed::from_client(embed))
        .then_retrieve(vector_store.clone())
        .then_transform_response(response_transformers::Summary::from_client(
            llm_client.clone(),
        ))
        .then_answer(answers::Simple::from_client(llm_client.clone()));

    let results: Vec<String> = pipeline
        .query_all(questions)
        .await?
        .iter()
        .map(|result| result.answer().to_string())
        .collect();

    Ok(results)
}
