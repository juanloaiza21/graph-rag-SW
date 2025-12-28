use mongodb::Client as MongoClient;
use neo4rs::Graph;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub mongo: MongoClient,
    pub graph: Arc<Graph>,
    pub gemini_key: String,
    pub mongo_db_name: String,
}

impl AppState {
    pub fn new(
        mongo: MongoClient,
        graph: Arc<Graph>,
        gemini_key: String,
        mongo_db_name: String,
    ) -> Self {
        Self {
            mongo,
            graph,
            gemini_key,
            mongo_db_name,
        }
    }
}
