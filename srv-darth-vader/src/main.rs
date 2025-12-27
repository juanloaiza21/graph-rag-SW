mod api;
mod models;
mod services;
mod state;
mod utils;

use dotenvy::dotenv;
use mongodb::{Client as MongoClient, options::ClientOptions};
use neo4rs::Graph;
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI required");
    let neo4j_uri = env::var("NEO4J_URI").expect("NEO4J_URI required");
    let neo4j_user = env::var("NEO4J_USER").expect("NEO4J_USER required");
    let neo4j_pass = env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD required");
    let gemini_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY required");
    let db_name = env::var("MONGO_DB_NAME").unwrap_or("starwars_db".to_string());

    let mongo_client = MongoClient::with_options(ClientOptions::parse(mongo_uri).await?)?;
    let graph = Arc::new(Graph::new(&neo4j_uri, &neo4j_user, &neo4j_pass).await?);

    let state = Arc::new(state::AppState {
        mongo: mongo_client,
        graph,
        gemini_key,
        mongo_db_name: db_name,
    });

    let app = api::app_router(state).layer(CorsLayer::permissive());

    let addr = "0.0.0.0:3000";
    println!("🚀 Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
