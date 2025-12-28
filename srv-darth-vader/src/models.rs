use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait GraphableSource {
    fn get_entity_id(&self) -> String;
    fn get_entity_label(&self) -> String;
    fn get_entity_name(&self) -> String;
    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value>;
    fn get_rich_text(&self) -> String;
    fn get_edges(&self) -> Vec<GraphEdge>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,                // "char_1"
    pub original_swapi_id: String, // "1"
    pub name: String,              // "Luke Skywalker"
    pub wiki_description: String,
    pub birth_year: String,
    pub gender: String,
    pub height: String,
    pub mass: String,
    pub homeworld_id: String,
    pub species_ids: Vec<String>,
    pub source: String,
    #[serde(default)]
    pub film_ids: Vec<String>,
    #[serde(default)]
    pub starship_ids: Vec<String>,
    #[serde(default)]
    pub vehicle_ids: Vec<String>,
}

impl GraphableSource for CharacterRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Character".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.name.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), Value::String(self.name.clone()));
        map.insert(
            "birth_year".to_string(),
            Value::String(self.birth_year.clone()),
        );
        map.insert("gender".to_string(), Value::String(self.gender.clone()));
        map.insert("height".to_string(), Value::String(self.height.clone()));
        map.insert("mass".to_string(), Value::String(self.mass.clone()));
        map.insert(
            "homeworld_id".to_string(),
            Value::String(self.homeworld_id.clone()),
        );
        map.insert(
            "species_ids".to_string(),
            Value::Array(
                self.species_ids
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert(
            "original_swapi_id".to_string(),
            Value::String(self.original_swapi_id.clone()),
        );
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));
        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_description.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let source_id = self.id.clone(); // Ej: "char_1"

        // 1. PLANET (BORN_ON)
        if !self.homeworld_id.is_empty() && self.homeworld_id != "unknown" {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Character".to_string(),
                target_id: format!("planet_{}", self.homeworld_id),
                relation_type: RelationType::BornOn.as_str().to_string(),
                target_label: "Planet".to_string(),
            });
        }

        // 2. SPECIES (BELONGS_TO)
        for species_id in &self.species_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Character".to_string(),
                target_id: format!("species_{}", species_id),
                relation_type: RelationType::BelongsTo.as_str().to_string(),
                target_label: "Species".to_string(),
            });
        }

        // --- LOS PUENTES QUE FALTABAN ---

        // 3. FILMS (APPEARED_IN)
        for film_id in &self.film_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Character".to_string(),
                target_id: format!("film_{}", film_id), // Prefijo film_
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        // 4. STARSHIPS (PILOTS)
        for ship_id in &self.starship_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Character".to_string(),
                target_id: format!("starship_{}", ship_id), // Prefijo starship_
                relation_type: RelationType::Pilots.as_str().to_string(),
                target_label: "Starship".to_string(),
            });
        }

        for vehicle_id in &self.vehicle_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Character".to_string(),
                target_id: format!("vehicle_{}", vehicle_id),
                relation_type: RelationType::Pilots.as_str().to_string(),
                target_label: "Vehicle".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoviesRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,
    pub title: String,
    pub episode_id: i32,
    pub director: String,
    pub release_date: String,
    pub opening_crawl: String,
    pub wiki_plot: String,
    pub character_ids: Vec<String>,
    pub source: String,
}

impl GraphableSource for MoviesRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Film".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.title.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("title".to_string(), Value::String(self.title.clone()));
        map.insert(
            "episode_id".to_string(),
            Value::Number(self.episode_id.into()),
        );
        map.insert("director".to_string(), Value::String(self.director.clone()));
        map.insert(
            "release_date".to_string(),
            Value::String(self.release_date.clone()),
        );
        map.insert(
            "opening_crawl".to_string(),
            Value::String(self.opening_crawl.clone()),
        );
        map.insert(
            "character_ids".to_string(),
            Value::Array(
                self.character_ids
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));
        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_plot.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let film_target_id = self.id.clone(); // Ej: "film_1"

        // 1. Relación: PERSONAJES -> PELÍCULA
        for char_raw_id in &self.character_ids {
            let char_node_id = format!("char_{}", char_raw_id);
            edges.push(GraphEdge {
                source_id: char_node_id,
                source_label: "Character".to_string(),
                target_id: film_target_id.clone(),
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanetRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,
    pub original_swapi_id: String,
    pub name: String,
    pub rotation_period: String,
    pub orbital_period: String,
    pub diameter: String,
    pub climate: String,
    pub gravity: String,
    pub terrain: String,
    pub surface_water: String,
    pub population: String,
    pub wiki_description: String,
    pub film_ids: Vec<String>,
    pub resident_ids: Vec<String>,
    pub source: String,
}

impl GraphableSource for PlanetRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Planet".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.name.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), Value::String(self.name.clone()));
        map.insert(
            "rotation_period".to_string(),
            Value::String(self.rotation_period.clone()),
        );
        map.insert(
            "orbital_period".to_string(),
            Value::String(self.orbital_period.clone()),
        );
        map.insert("diameter".to_string(), Value::String(self.diameter.clone()));
        map.insert("climate".to_string(), Value::String(self.climate.clone()));
        map.insert("gravity".to_string(), Value::String(self.gravity.clone()));
        map.insert("terrain".to_string(), Value::String(self.terrain.clone()));
        map.insert(
            "surface_water".to_string(),
            Value::String(self.surface_water.clone()),
        );
        map.insert(
            "population".to_string(),
            Value::String(self.population.clone()),
        );
        map.insert(
            "film_ids".to_string(),
            Value::Array(self.film_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert(
            "resident_ids".to_string(),
            Value::Array(
                self.resident_ids
                    .iter()
                    .cloned()
                    .map(Value::String)
                    .collect(),
            ),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert(
            "original_swapi_id".to_string(),
            Value::String(self.original_swapi_id.clone()),
        );
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));
        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_description.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let source_id = self.id.clone(); // "planet_X"

        for film_id in &self.film_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Planet".to_string(),
                target_id: format!("film_{}", film_id), // <--- FIX: Prefijo film_
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpeciesRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,
    pub original_swapi_id: String,
    pub name: String,
    pub classification: Option<String>,
    pub designation: String,
    pub average_height: String,
    pub average_lifespan: String,
    pub language: String,
    pub skin_colors: String,
    pub wiki_description: String,
    pub homeworld_id: Option<String>,
    pub people_ids: Vec<String>,
    pub film_ids: Vec<String>,
    pub source: String,
}

impl GraphableSource for SpeciesRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Species".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.name.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();

        map.insert("name".to_string(), Value::String(self.name.clone()));
        map.insert(
            "designation".to_string(),
            Value::String(self.designation.clone()),
        );
        map.insert(
            "average_height".to_string(),
            Value::String(self.average_height.clone()),
        );
        map.insert(
            "average_lifespan".to_string(),
            Value::String(self.average_lifespan.clone()),
        );
        map.insert("language".to_string(), Value::String(self.language.clone()));
        map.insert(
            "skin_colors".to_string(),
            Value::String(self.skin_colors.clone()),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert(
            "original_swapi_id".to_string(),
            Value::String(self.original_swapi_id.clone()),
        );
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));

        map.insert(
            "people_ids".to_string(),
            Value::Array(self.people_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert(
            "film_ids".to_string(),
            Value::Array(self.film_ids.iter().cloned().map(Value::String).collect()),
        );

        map.insert(
            "classification".to_string(),
            Value::String(
                self.classification
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        );

        map.insert(
            "homeworld_id".to_string(),
            Value::String(
                self.homeworld_id
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
            ),
        );

        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_description.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let source_id = self.id.clone(); // "species_X"

        if let Some(homeworld) = &self.homeworld_id {
            if homeworld != "unknown" {
                edges.push(GraphEdge {
                    source_id: source_id.clone(),
                    source_label: "Species".to_string(),
                    target_id: format!("planet_{}", homeworld), // <--- FIX: Prefijo planet_
                    relation_type: RelationType::ResidentOf.as_str().to_string(),
                    target_label: "Planet".to_string(),
                });
            }
        }

        // Relación: APPEARED_IN (Species -> Film)
        for film_id in &self.film_ids {
            edges.push(GraphEdge {
                source_id: source_id.clone(),
                source_label: "Species".to_string(),
                target_id: format!("film_{}", film_id), // <--- FIX: Prefijo film_
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StarshipRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,
    pub original_swapi_id: String,
    pub name: String,
    pub model: String,
    pub manufacturer: String,
    pub wiki_description: String,
    pub cost_in_credits: String,
    pub length: String,
    pub max_atmosphering_speed: String,
    pub crew: String,
    pub passengers: String,
    pub cargo_capacity: String,
    pub hyperdrive_rating: String,
    pub starship_class: String,
    pub pilot_ids: Vec<String>,
    pub film_ids: Vec<String>,
    pub source: String,
}

impl GraphableSource for StarshipRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Starship".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.name.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), Value::String(self.name.clone()));
        map.insert("model".to_string(), Value::String(self.model.clone()));
        map.insert(
            "manufacturer".to_string(),
            Value::String(self.manufacturer.clone()),
        );
        map.insert(
            "cost_in_credits".to_string(),
            Value::String(self.cost_in_credits.clone()),
        );
        map.insert("length".to_string(), Value::String(self.length.clone()));
        map.insert(
            "max_atmosphering_speed".to_string(),
            Value::String(self.max_atmosphering_speed.clone()),
        );
        map.insert("crew".to_string(), Value::String(self.crew.clone()));
        map.insert(
            "passengers".to_string(),
            Value::String(self.passengers.clone()),
        );
        map.insert(
            "cargo_capacity".to_string(),
            Value::String(self.cargo_capacity.clone()),
        );
        map.insert(
            "hyperdrive_rating".to_string(),
            Value::String(self.hyperdrive_rating.clone()),
        );
        map.insert(
            "starship_class".to_string(),
            Value::String(self.starship_class.clone()),
        );
        map.insert(
            "pilot_ids".to_string(),
            Value::Array(self.pilot_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert(
            "film_ids".to_string(),
            Value::Array(self.film_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert(
            "original_swapi_id".to_string(),
            Value::String(self.original_swapi_id.clone()),
        );
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));
        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_description.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let ship_id = self.id.clone(); // "starship_X"

        // Relación: PILOTS (Character -> Starship)
        for pilot_id in &self.pilot_ids {
            edges.push(GraphEdge {
                source_id: format!("char_{}", pilot_id), // <--- FIX: El piloto es un char_
                source_label: "Character".to_string(),
                target_id: ship_id.clone(),
                relation_type: RelationType::Pilots.as_str().to_string(),
                target_label: "Starship".to_string(),
            });
        }

        // Relación: APPEARED_IN (Starship -> Film)
        for film_id in &self.film_ids {
            edges.push(GraphEdge {
                source_id: ship_id.clone(),
                source_label: "Starship".to_string(),
                target_id: format!("film_{}", film_id), // <--- FIX: Prefijo film_
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VehicleRaw {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub id: String,
    pub original_swapi_id: String,
    pub name: String,
    pub model: String,
    pub manufacturer: String,
    pub wiki_description: String,
    pub cost_in_credits: String,
    pub length: String,
    pub max_atmosphering_speed: String,
    pub crew: String,
    pub passengers: String,
    pub cargo_capacity: String,
    pub vehicle_class: String,
    pub pilot_ids: Vec<String>,
    pub film_ids: Vec<String>,
    pub source: String,
}

impl GraphableSource for VehicleRaw {
    fn get_entity_id(&self) -> String {
        self.id.clone()
    }

    fn get_entity_label(&self) -> String {
        "Vehicle".to_string()
    }

    fn get_entity_name(&self) -> String {
        self.name.clone()
    }

    fn get_metadata_as_map(&self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("name".to_string(), Value::String(self.name.clone()));
        map.insert("model".to_string(), Value::String(self.model.clone()));
        map.insert(
            "manufacturer".to_string(),
            Value::String(self.manufacturer.clone()),
        );
        map.insert(
            "cost_in_credits".to_string(),
            Value::String(self.cost_in_credits.clone()),
        );
        map.insert("length".to_string(), Value::String(self.length.clone()));
        map.insert(
            "max_atmosphering_speed".to_string(),
            Value::String(self.max_atmosphering_speed.clone()),
        );
        map.insert("crew".to_string(), Value::String(self.crew.clone()));
        map.insert(
            "passengers".to_string(),
            Value::String(self.passengers.clone()),
        );
        map.insert(
            "cargo_capacity".to_string(),
            Value::String(self.cargo_capacity.clone()),
        );
        map.insert(
            "vehicle_class".to_string(),
            Value::String(self.vehicle_class.clone()),
        );
        map.insert(
            "pilot_ids".to_string(),
            Value::Array(self.pilot_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert(
            "film_ids".to_string(),
            Value::Array(self.film_ids.iter().cloned().map(Value::String).collect()),
        );
        map.insert("source".to_string(), Value::String(self.source.clone()));
        map.insert(
            "original_swapi_id".to_string(),
            Value::String(self.original_swapi_id.clone()),
        );
        map.insert("original_oid".to_string(), Value::String(self._id.to_hex()));
        map
    }

    fn get_rich_text(&self) -> String {
        self.wiki_description.clone()
    }

    fn get_edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let ship_id = self.id.clone(); // "Vehicle_X"

        // Relación: PILOTS (Character -> Vehicle)
        for pilot_id in &self.pilot_ids {
            edges.push(GraphEdge {
                source_id: format!("char_{}", pilot_id), // <--- FIX: El piloto es un char_
                source_label: "Character".to_string(),
                target_id: ship_id.clone(),
                relation_type: RelationType::Pilots.as_str().to_string(),
                target_label: "Vehicle".to_string(),
            });
        }

        // Relación: APPEARED_IN (Vehicle -> Film)
        for film_id in &self.film_ids {
            edges.push(GraphEdge {
                source_id: ship_id.clone(),
                source_label: "Vehicle".to_string(),
                target_id: format!("film_{}", film_id), // <--- FIX: Prefijo film_
                relation_type: RelationType::AppearedIn.as_str().to_string(),
                target_label: "Film".to_string(),
            });
        }

        edges
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source_id: String,
    pub source_label: String,
    pub target_id: String,
    pub relation_type: String,
    pub target_label: String,
}

pub enum RelationType {
    AppearedIn,
    BornOn,
    BelongsTo,
    Pilots,
    Produced,
    ResidentOf,
}

impl RelationType {
    fn as_str(&self) -> &'static str {
        match self {
            RelationType::AppearedIn => "APPEARED_IN",
            RelationType::BornOn => "BORN_ON",
            RelationType::BelongsTo => "BELONGS_TO",
            RelationType::Pilots => "PILOTS",
            RelationType::Produced => "PRODUCED",
            RelationType::ResidentOf => "RESIDENT_OF",
        }
    }
}
