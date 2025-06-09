use swiftide::{
    indexing::EmbeddedField,
    integrations::{self, openai::GenericOpenAI, pgvector::PgVector},
    query::{self, answers, query_transformers, response_transformers},
};

#[allow(unused)]
pub struct VectorStore {
    pub vector_store: PgVector,
    // pub embed: Embed,
    pub llm_client: GenericOpenAI,
}

impl VectorStore {
    pub async fn try_new(
        table_name: &str,
        metadata: &str,
        db_url: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pgv_storage = PgVector::builder()
            .db_url(db_url)
            .vector_size(1536)
            .with_vector(EmbeddedField::Combined)
            .with_metadata(metadata)
            .table_name(table_name)
            .build()
            .unwrap();
        let openai_client = integrations::openai::OpenAI::builder()
            .default_embed_model("text-embedding-3-small")
            .default_prompt_model("gpt-4o")
            .build()?;

        // let embed = Embed::new(openai_client.clone());

        Self::new(pgv_storage, openai_client)
    }
    pub fn new(
        vector_store: PgVector,

        llm_client: GenericOpenAI,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            vector_store,

            llm_client,
        })
    }
}

pub async fn ask_query(
    llm_client: GenericOpenAI,
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
        .then_transform_query(query_transformers::Embed::from_client(llm_client.clone()))
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
