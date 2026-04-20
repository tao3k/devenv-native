use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_wendao_core::{Entity, EntityType, Relation, RelationType};

pub(super) fn build_promotion_entity(context: &Value, decision: &str) -> Entity {
    let query = context
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("memory promotion");
    let summary = context
        .get("annotated_prompt")
        .and_then(Value::as_str)
        .unwrap_or("promotion context unavailable");
    let memory_id = context
        .get("memory_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(generate_fallback_memory_id, ToString::to_string);
    let title = context
        .get("memory_title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || format!("Memory Promotion {memory_id}"),
            ToString::to_string,
        );

    let description =
        format!("MemRL promotion decision={decision}; query={query}; summary={summary}");

    Entity::new(
        format!("memory:{memory_id}"),
        title,
        EntityType::Document,
        description,
    )
    .with_source(Some("qianji://memory_promotion".to_string()))
    .with_metadata("memory_id".to_string(), Value::String(memory_id))
    .with_metadata(
        "promotion_decision".to_string(),
        Value::String(decision.to_string()),
    )
    .with_metadata("query".to_string(), Value::String(query.to_string()))
}

pub(super) fn build_topic_entity(context: &Value) -> Entity {
    let query = context
        .get("query")
        .and_then(Value::as_str)
        .unwrap_or("memory promotion");
    let topic_key = normalize_topic_key(query);
    Entity::new(
        format!("topic:{topic_key}"),
        format!("Topic {topic_key}"),
        EntityType::Concept,
        format!("Promotion topic derived from query: {query}"),
    )
    .with_source(Some("qianji://memory_promotion".to_string()))
    .with_metadata("topic_query".to_string(), Value::String(query.to_string()))
}

pub(super) fn build_promotion_relation(
    source: &Entity,
    topic: &Entity,
    decision: &str,
) -> Relation {
    Relation::new(
        source.name.clone(),
        topic.name.clone(),
        RelationType::RelatedTo,
        "Memory promotion linkage".to_string(),
    )
    .with_source_doc(Some("qianji://memory_promotion".to_string()))
    .with_metadata(
        "promotion_decision".to_string(),
        Value::String(decision.to_string()),
    )
}

fn normalize_topic_key(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len().min(64));
    let mut previous_was_separator = false;

    for ch in raw.chars() {
        if normalized.len() >= 64 {
            break;
        }
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            normalized.push(lower);
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('-');
            previous_was_separator = true;
        }
    }

    let trimmed = normalized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "general".to_string()
    } else {
        trimmed
    }
}

fn generate_fallback_memory_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    format!("auto-{millis}")
}
