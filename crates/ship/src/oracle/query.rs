use anyhow::Result;
use crate::oracle::store::OracleStore;
use crate::oracle::vector_store::VectorStore;
use crate::oracle::embed::MiniLM;

pub struct QueryEngine {
    _store: OracleStore,
    vector_store: VectorStore,
    model: MiniLM,
}

#[derive(Debug)]
pub struct QueryResult {
    pub name: String,
    pub path: String,
    pub score: f32,
    pub snippet: String,
}

impl QueryEngine {
    pub async fn new(db_path: &str, vector_uri: &str) -> Result<Self> {
        let store = OracleStore::open(db_path)?;
        let vector_store = VectorStore::new(vector_uri).await?;
        let model = MiniLM::new()?;
        
        Ok(Self {
            store,
            vector_store,
            model,
        })
    }

    pub async fn query(&mut self, text: &str) -> Result<Vec<QueryResult>> {
        // 1. Embed query
        let vec = self.model.embed(text)?;
        
        // 2. Vector Search
        let _hits = self.vector_store.search(vec, 5).await?;
        
        // 3. Enrich with Graph Data
        let results = Vec::new();
        // TODO: Map ID back to node details via store.
        // For MVP skeleton: return empty or mock
        
        Ok(results)
    }
}
