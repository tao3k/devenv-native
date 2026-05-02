use super::{
    OpenApiFileRestDocsRulePack, build_rest_docs_collection_context,
    build_rest_docs_contract_suite, run_and_persist_rest_docs_contract_feedback,
    run_rest_docs_contract_feedback,
};
use crate::contract_feedback::{
    CollectionContext, ContractRunConfig, NoopAdvisoryAuditExecutor, RulePack,
};
use crate::sovereign::InMemoryContractFeedbackSink;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn write_openapi_yaml(temp_dir: &TempDir) -> PathBuf {
    let path = temp_dir.path().join("openapi.yaml");
    let content = r#"
openapi: 3.1.0
paths:
  /api/search:
    get:
      responses:
        "200":
          description: ok
"#;
    must_ok(
        fs::write(&path, content),
        "should write temporary OpenAPI fixture",
    );
    path
}

#[test]
fn openapi_file_rest_docs_rule_pack_collects_yaml_document() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_openapi_yaml(&temp_dir);
    let pack = OpenApiFileRestDocsRulePack::new(&openapi_path);

    let artifacts = must_ok(
        pack.collect(&CollectionContext::default()),
        "file-backed rest_docs collect should succeed",
    );

    assert_eq!(artifacts.len(), 1);
    assert_eq!(
        artifacts.artifacts[0].path.as_deref(),
        Some(openapi_path.as_path())
    );
    assert_eq!(
        artifacts.artifacts[0].content["openapi"],
        serde_json::Value::String("3.1.0".to_string())
    );
}

#[tokio::test]
async fn run_and_persist_rest_docs_contract_feedback_uses_sink() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_openapi_yaml(&temp_dir);
    let ctx =
        build_rest_docs_collection_context(&openapi_path, Some(temp_dir.path().to_path_buf()));
    let sink = InMemoryContractFeedbackSink::new();

    let result = must_ok(
        run_and_persist_rest_docs_contract_feedback(
            &openapi_path,
            ctx,
            &ContractRunConfig::default(),
            &sink,
        )
        .await,
        "run-and-persist rest_docs contract feedback should succeed",
    );

    assert_eq!(
        result.run.report.suite_id,
        "qianji-rest-docs-contract-feedback"
    );
    assert_eq!(result.persisted_entry_ids.len(), 2);
    assert_eq!(sink.len(), 2);
    assert_eq!(
        sink.entries()
            .into_iter()
            .map(|entry| entry.id)
            .collect::<Vec<_>>(),
        result.persisted_entry_ids
    );
}

#[tokio::test]
async fn run_rest_docs_contract_feedback_returns_deterministic_report() {
    let temp_dir = must_ok(TempDir::new(), "should create temp dir");
    let openapi_path = write_openapi_yaml(&temp_dir);
    let ctx =
        build_rest_docs_collection_context(&openapi_path, Some(temp_dir.path().to_path_buf()));

    let result = must_ok(
        run_rest_docs_contract_feedback(
            &openapi_path,
            ctx,
            &ContractRunConfig::default(),
            &NoopAdvisoryAuditExecutor,
        )
        .await,
        "rest_docs contract feedback should succeed without persistence",
    );

    assert_eq!(result.report.suite_id, "qianji-rest-docs-contract-feedback");
    assert_eq!(result.report.stats.total, 2);
    assert_eq!(result.knowledge_entries.len(), 2);
}

#[test]
fn build_rest_docs_contract_suite_registers_one_pack() {
    let suite = build_rest_docs_contract_suite("openapi.yaml");
    assert_eq!(suite.id(), "qianji-rest-docs-contract-feedback");
    assert_eq!(suite.rule_pack_count(), 1);
}
