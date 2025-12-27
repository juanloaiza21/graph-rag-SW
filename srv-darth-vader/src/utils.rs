// src/utils.rs
use neo4rs::{BoltMap, BoltString, BoltType};
use serde_json::Value;

pub fn bolt_map_from_serde(map: serde_json::Map<String, Value>) -> BoltMap {
    let mut bolt_map = BoltMap {
        value: Default::default(),
    };

    for (k, v) in map {
        let key: BoltString = k.into();

        let val: Option<BoltType> = match v {
            Value::String(s) => Some(s.into()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(i.into())
                } else if let Some(f) = n.as_f64() {
                    Some(f.into())
                } else {
                    None
                }
            }
            Value::Bool(b) => Some(b.into()),
            _ => None,
        };

        if let Some(bolt_val) = val {
            bolt_map.value.insert(key, bolt_val);
        }
    }
    bolt_map
}
