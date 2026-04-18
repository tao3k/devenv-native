use serde::Serialize;
use serde_json::{Map, Value};

pub(crate) fn canonical_json<T: Serialize>(value: T) -> Value {
    let Ok(value) = serde_json::to_value(value) else {
        panic!("snapshot payload should serialize to JSON");
    };
    canonicalize_value(value)
}

fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        Value::Object(map) => Value::Object(canonicalize_map(map)),
        value => value,
    }
}

fn canonicalize_map(map: Map<String, Value>) -> Map<String, Value> {
    let mut entries: Vec<_> = map.into_iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    entries
        .into_iter()
        .map(|(key, value)| (key, canonicalize_value(value)))
        .collect()
}
