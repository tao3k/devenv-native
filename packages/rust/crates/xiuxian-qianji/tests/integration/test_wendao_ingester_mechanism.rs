//! Wendao ingester mechanism integration tests.

#![cfg(feature = "wendao-integration")]

use serde_json::json;
use xiuxian_qianji::contracts::QianjiMechanism;
use xiuxian_qianji::executors::WendaoIngesterMechanism;
use xiuxian_wendao_core::{EntityType, RelationType};

#[tokio::test]
async fn wendao_ingester_emits_structured_entity_without_persistence() {
    let mechanism = WendaoIngesterMechanism {
        output_key: "promotion_entity".to_string(),
        graph_scope: Some("qianji:test_scope".to_string()),
        graph_scope_key: None,
        graph_dimension: 256,
        persist: false,
        persist_best_effort: true,
    };

    let output = mechanism
        .execute(&json!({
            "selected_route": "Promote",
            "memory_id": "fix-race-condition",
            "memory_title": "Fix race condition in router index cache",
            "query": "router cache race",
            "annotated_prompt": "validated reflection payload"
        }))
        .await
        .unwrap_or_else(|error| panic!("wendao ingester should execute: {error}"));

    assert_eq!(output.data["promotion_decision"], "promote");
    assert_eq!(output.data["promotion_graph_scope"], "qianji:test_scope");
    assert_eq!(output.data["promotion_persisted"], false);

    let entity = &output.data["promotion_entity"];
    assert_eq!(entity["id"], "memory:fix-race-condition");
    assert_eq!(
        serde_json::from_value::<EntityType>(entity["entity_type"].clone()).ok(),
        Some(EntityType::Document)
    );
    assert_eq!(
        entity["metadata"]["promotion_decision"],
        serde_json::Value::String("promote".to_string())
    );
    let topic = &output.data["promotion_topic_entity"];
    assert_eq!(
        serde_json::from_value::<EntityType>(topic["entity_type"].clone()).ok(),
        Some(EntityType::Concept)
    );
    let relation = &output.data["promotion_relation"];
    assert_eq!(
        serde_json::from_value::<RelationType>(relation["relation_type"].clone()).ok(),
        Some(RelationType::RelatedTo)
    );
}

#[tokio::test]
async fn wendao_ingester_best_effort_records_persistence_error() {
    if std::env::var_os("XIUXIAN_WENDAO_GRAPH_VALKEY_URL").is_some()
        || std::env::var_os("VALKEY_URL").is_some()
    {
        eprintln!("skipping missing-Valkey best-effort assertion because Valkey is configured");
        return;
    }

    let mechanism = WendaoIngesterMechanism {
        output_key: "promotion_entity".to_string(),
        graph_scope: Some("qianji:test_scope_best_effort".to_string()),
        graph_scope_key: None,
        graph_dimension: 256,
        persist: true,
        persist_best_effort: true,
    };

    let output = mechanism
        .execute(&json!({
            "selected_route": "Promote",
            "memory_id": "missing-valkey",
            "query": "persistence unavailable",
            "annotated_prompt": "best effort fallback should still continue"
        }))
        .await
        .unwrap_or_else(|error| panic!("best-effort mode should not fail node: {error}"));

    assert_eq!(output.data["promotion_decision"], "promote");
    assert_eq!(output.data["promotion_persisted"], false);
    assert!(
        output
            .data
            .get("promotion_persist_error")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
    );
}
