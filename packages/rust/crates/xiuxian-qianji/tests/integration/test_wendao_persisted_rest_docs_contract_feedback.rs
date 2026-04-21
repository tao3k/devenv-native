//! Integration coverage for persisted Qianji to Wendao contract feedback.

use std::fs;
use std::path::PathBuf;

#[path = "support/workspace.rs"]
mod workspace;
use serde_json::Value;
use tempfile::TempDir;
use xiuxian_qianji::contract_feedback::{
    build_rest_docs_collection_context, run_and_persist_rest_docs_contract_feedback,
};
use xiuxian_qianji::sovereign::InMemoryContractFeedbackSink;
use xiuxian_testing::{ContractExecutionMode, ContractRunConfig};
use xiuxian_wendao_core::KnowledgeCategory;
use xiuxian_wendao_runtime::artifacts::openapi::load_bundled_wendao_gateway_openapi_document;

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn write_drifted_openapi_fixture(temp_dir: &TempDir) -> PathBuf {
    let mut document = must_ok(
        load_bundled_wendao_gateway_openapi_document(),
        "bundled Wendao OpenAPI document should parse",
    );
    let Some(content) = document
        .pointer_mut("/paths/~1api~1repo~1index/post/requestBody/content/application~1json")
        .and_then(Value::as_object_mut)
    else {
        panic!("bundled Wendao OpenAPI document should expose POST /api/repo/index JSON content");
    };
    content.insert(
        "schema".to_string(),
        serde_json::json!({
            "type": "object",
            "properties": {
                "repo": { "type": "string" },
                "refresh": { "type": "boolean" }
            },
            "required": ["repo"]
        }),
    );
    let removed = content.remove("example");
    assert!(
        removed.is_some(),
        "bundled Wendao OpenAPI document should include a POST /api/repo/index example"
    );

    let openapi_path = temp_dir.path().join("wendao_gateway.drifted.openapi.json");
    let rendered = must_ok(
        serde_json::to_vec_pretty(&document),
        "drifted OpenAPI document should serialize",
    );
    must_ok(
        fs::write(&openapi_path, rendered),
        "drifted OpenAPI document should write to temp file",
    );
    openapi_path
}

#[tokio::test]
async fn persisted_rest_docs_contract_feedback_persists_wendao_entries_through_sink() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_drifted_openapi_fixture(&temp_dir);
    let openapi_source = openapi_path.to_string_lossy().into_owned();
    let sink = InMemoryContractFeedbackSink::new();

    let result = must_ok(
        run_and_persist_rest_docs_contract_feedback(
            &openapi_path,
            build_rest_docs_collection_context(&openapi_path, Some(workspace::workspace_root())),
            &ContractRunConfig {
                execution_mode: ContractExecutionMode::Strict,
                generated_at: "2026-03-19T00:00:00Z".to_string(),
                ..ContractRunConfig::default()
            },
            &sink,
        )
        .await,
        "persisted rest_docs contract feedback should succeed for the drifted Wendao artifact",
    );

    assert_eq!(
        result.run.report.suite_id,
        "qianji-rest-docs-contract-feedback"
    );
    assert_eq!(result.run.report.stats.total, 1);
    assert_eq!(result.run.knowledge_batch.entries.len(), 1);
    assert_eq!(result.run.knowledge_entries.len(), 1);
    assert_eq!(result.persisted_entry_ids.len(), 1);
    assert_eq!(sink.len(), 1);

    let entry = &result.run.knowledge_entries[0];
    assert_eq!(entry.id, result.persisted_entry_ids[0]);
    assert_eq!(entry.category, KnowledgeCategory::Reference);
    assert_eq!(entry.source.as_deref(), Some(openapi_source.as_str()));
    assert!(entry.tags.iter().any(|tag| tag == "contract_feedback"));
    assert!(entry.tags.iter().any(|tag| tag == "pack:rest_docs"));
    assert!(entry.tags.iter().any(|tag| tag == "category:reference"));
    assert!(
        entry
            .metadata
            .get("rule_id")
            .is_some_and(|value| value == "REST-R007")
    );

    let persisted = sink.entries();
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].id, entry.id);
    assert_eq!(persisted[0].title, entry.title);
    assert_eq!(
        persisted[0].source.as_deref(),
        Some(openapi_source.as_str())
    );
}

xiuxian_testing::crate_test_policy_harness!();
