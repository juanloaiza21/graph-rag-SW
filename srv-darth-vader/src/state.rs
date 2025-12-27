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
