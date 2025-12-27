use crate::{
    models::{CharacterRaw, MoviesRaw, PlanetRaw, SpeciesRaw, StarshipRaw, VehicleRaw},
    services::process_collection,
    state::AppState,
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::Arc;

pub fn app_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ingest/{collection}", post(ingest_handler))
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "srv-darth-vader" }))
}

async fn ingest_handler(
    Path(collection): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let col_clone = collection.clone();
    let state_clone = state.clone();

    // Background Task
    tokio::spawn(async move {
        let result = match col_clone.as_str() {
            "characters_raw" => process_collection::<CharacterRaw>(&col_clone, state_clone).await,
            "movies_raw" => process_collection::<MoviesRaw>(&col_clone, state_clone).await,
            "planets_raw" => process_collection::<PlanetRaw>(&col_clone, state_clone).await,
            "species_raw" => process_collection::<SpeciesRaw>(&col_clone, state_clone).await,
            "starships_raw" => process_collection::<StarshipRaw>(&col_clone, state_clone).await,
            "vehicles_raw" => process_collection::<VehicleRaw>(&col_clone, state_clone).await,
            _ => Err(anyhow::anyhow!("Colección no mapeada")),
        };

        if let Err(e) = result {
            eprintln!("Job failed for {}: {:?}", col_clone, e);
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "pending",
            "message": format!("Ingesta iniciada para {}", collection)
        })),
    )
}
