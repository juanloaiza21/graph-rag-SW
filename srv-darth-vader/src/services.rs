use crate::{models::GraphableSource, state::AppState, utils::bolt_map_from_serde};
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use neo4rs::{BoltType, query};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;

// --- GEMINI CLIENT LOGIC ---
pub async fn get_gemini_embedding(
    client: &reqwest::Client,
    key: &str,
    text: &str,
) -> Result<Vec<f32>> {
    if key.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "CRÍTICO: La Google API Key está vacía. Revisa tu .env"
        ));
    }

    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent";

    if text.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Texto vacío, no se puede generar embedding"
        ));
    }

    let res = client
        .post(url)
        .header("x-goog-api-key", key) // <--- ESTO ES LA CLAVE
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "content": {
                "parts": [{ "text": text }]
            }
        }))
        .send()
        .await?;

    if !res.status().is_success() {
        let error_text = res.text().await?;
        return Err(anyhow::anyhow!("Gemini API Error: {}", error_text));
    }

    let json: Value = res.json().await?;

    let values = json["embedding"]["values"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Formato respuesta Gemini inválido: {:?}", json))?
        .iter()
        .map(|v| v.as_f64().unwrap_or(0.0) as f32)
        .collect();

    Ok(values)
}

pub async fn process_collection<T>(collection_name: &str, state: Arc<AppState>) -> Result<()>
where
    T: DeserializeOwned + GraphableSource + Send + Sync + Unpin,
{
    println!(">>> Procesando colección: {}", collection_name);
    let collection = state
        .mongo
        .database(&state.mongo_db_name)
        .collection::<T>(collection_name);
    let mut cursor = collection.find(doc! {}).await?;
    let http_client = reqwest::Client::new();

    while let Some(doc) = cursor.try_next().await? {
        let entity_id = doc.get_entity_id();
        let label = doc.get_entity_label();

        let mut metadata = doc.get_metadata_as_map();
        metadata.insert("id".to_string(), Value::String(entity_id.clone()));

        let entity_query = format!("MERGE (e:{} {{id: $id}}) SET e += $props", label);

        let params = query(&entity_query)
            .param("id", entity_id.clone())
            .param("props", BoltType::Map(bolt_map_from_serde(metadata)));

        if let Err(e) = state.graph.run(params).await {
            eprintln!("Error insertando entidad {}: {}", entity_id, e);
            continue;
        }

        // Embeddings
        let raw_text = doc.get_rich_text();
        if raw_text.len() < 10 {
            continue;
        }

        let context_text = format!("About {} ({}): {}", doc.get_entity_name(), label, raw_text);

        match get_gemini_embedding(&http_client, &state.gemini_key, &context_text).await {
            Ok(vector) => {
                let chunk_query = format!(
                    "
                    MATCH (e:{label} {{id: $eid}})
                    CREATE (c:Chunk {{
                        text: $text, embedding: $vec, source: $coll, createdAt: datetime()
                    }})
                    MERGE (e)-[:HAS_CONTEXT]->(c)
                ",
                    label = label
                );

                let chunk_params = query(&chunk_query)
                    .param("eid", entity_id)
                    .param("text", context_text)
                    .param("vec", vector)
                    .param("coll", collection_name);

                if let Err(e) = state.graph.run(chunk_params).await {
                    eprintln!("Error insertando chunk: {}", e);
                }
            }
            Err(e) => eprintln!("Error generando embedding: {}", e),
        }
    }
    println!("<<< Finalizado: {}", collection_name);
    Ok(())
}
