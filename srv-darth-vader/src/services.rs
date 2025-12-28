use crate::{models::GraphableSource, state::AppState, utils::bolt_map_from_serde};
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use neo4rs::{BoltMap, BoltNull, BoltString, BoltType, Graph, query};
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
        let raw_text = doc.get_rich_text();
        let entity_name = doc.get_entity_name();

        let embedding_vector = if raw_text.len() > 5 {
            let context_text = format!("About {}: {}", entity_name, raw_text);
            match get_gemini_embedding(&http_client, &state.gemini_key, &context_text).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("⚠️ Error embedding para {}: {}", entity_name, e);
                    vec![]
                }
            }
        } else {
            vec![]
        };

        if let Err(e) = ingest_entity_to_graph(&state.graph, &doc, embedding_vector).await {
            eprintln!("❌ Error insertando en Neo4j ({}): {}", entity_name, e);
        } else {
            println!("✅ Ingestado: {}", entity_name);
        }
    }

    println!("<<< Finalizado: {}", collection_name);
    Ok(())
}

fn json_map_to_bolt_type(map: serde_json::Map<String, Value>) -> BoltType {
    let mut bolt_map = BoltMap::new();

    for (key, value) in map {
        let bolt_key: BoltString = key.into();
        let bolt_val: BoltType = match value {
            Value::String(s) => s.into(),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i.into()
                } else if let Some(f) = n.as_f64() {
                    f.into()
                } else {
                    n.to_string().into()
                }
            }
            Value::Bool(b) => b.into(),
            Value::Null => BoltType::Null(BoltNull),

            _ => value.to_string().into(),
        };
        bolt_map.put(bolt_key, bolt_val);
    }
    BoltType::Map(bolt_map)
}

pub async fn ingest_entity_to_graph<T>(
    graph: &Arc<Graph>,
    entity: &T,
    embedding_vector: Vec<f32>,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: GraphableSource + Sync + Send,
{
    let label = entity.get_entity_label();
    let id = entity.get_entity_id();
    let name = entity.get_entity_name();

    let props_map = entity.get_metadata_as_map();
    let props_bolt = json_map_to_bolt_type(props_map);

    let node_query_str = format!(
        "MERGE (n:{label} {{id: $id}}) 
         SET n += $props, 
             n.embedding = $vector,
             n.name = $name,
             n.last_updated = timestamp()"
    );

    let node_query = query(&node_query_str)
        .param("id", id)
        .param("vector", embedding_vector)
        .param("name", name)
        .param("props", props_bolt);

    graph.run(node_query).await?;

    let edges = entity.get_edges();

    for edge in edges {
        let edge_query_str = format!(
            "MERGE (source:{source_label} {{id: $source_id}})
             MERGE (target:{target_label} {{id: $target_id}})
             MERGE (source)-[r:{relation}]->(target)",
            source_label = edge.source_label,
            target_label = edge.target_label,
            relation = edge.relation_type
        );

        let edge_query = query(&edge_query_str)
            .param("source_id", edge.source_id)
            .param("target_id", edge.target_id);

        if let Err(e) = graph.run(edge_query).await {
            eprintln!(
                "⚠️ Warning edge ({} -> {}): {:?}",
                label, edge.target_label, e
            );
        }
    }

    Ok(())
}
