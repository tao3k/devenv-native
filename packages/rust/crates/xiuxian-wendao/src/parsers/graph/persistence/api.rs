//! `parsers::graph::persistence::api` owns Wendao graph persistence api behavior.

use super::entity::parse_entity_type_str;
use super::relation::parse_persisted_relation_type;
use crate::entity::{Entity, Relation};
use serde_json::Value;

/// Create an [`Entity`] from a JSON dict.
#[must_use]
pub fn entity_from_dict(data: &Value) -> Option<Entity> {
    let name = data.get("name")?.as_str()?.to_string();
    let entity_type = parse_entity_type_str(data.get("entity_type")?.as_str()?);
    let description = data
        .get("description")
        .map(|value| value.as_str().unwrap_or("").to_string())
        .unwrap_or_default();

    let id = format!(
        "{}:{}",
        entity_type.to_string().to_lowercase(),
        name.to_lowercase().replace(' ', "_")
    );

    let entity = Entity::new(id, name, entity_type, description)
        .with_source(
            data.get("source")
                .and_then(|value| value.as_str().map(str::to_string)),
        )
        .with_aliases(string_list_from_value(data.get("aliases")).unwrap_or_default())
        .with_confidence(f32_from_value(data.get("confidence")).unwrap_or(1.0));

    Some(entity)
}

/// Create a [`Relation`] from a JSON dict.
#[must_use]
pub fn relation_from_dict(data: &Value) -> Option<Relation> {
    let source = data.get("source")?.as_str()?.to_string();
    let target = data.get("target")?.as_str()?.to_string();
    let relation_type = parse_persisted_relation_type(data.get("relation_type")?.as_str()?);
    let description = data
        .get("description")
        .map(|value| value.as_str().unwrap_or("").to_string())
        .unwrap_or_default();

    let relation = Relation::new(source, target, relation_type, description)
        .with_source_doc(
            data.get("source_doc")
                .and_then(|value| value.as_str().map(str::to_string)),
        )
        .with_confidence(f32_from_value(data.get("confidence")).unwrap_or(1.0));

    Some(relation)
}

fn string_list_from_value(value: Option<&Value>) -> Option<Vec<String>> {
    value.and_then(|value| {
        value.as_array().map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
    })
}

fn f32_from_value(value: Option<&Value>) -> Option<f32> {
    value.and_then(|value| serde_json::from_value::<f32>(value.clone()).ok())
}
