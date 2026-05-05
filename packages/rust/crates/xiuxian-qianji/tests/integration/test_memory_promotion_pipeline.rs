//! Memory promotion pipeline integration tests.

use std::sync::Arc;

use serde_json::json;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::{MEMORY_PROMOTION_PIPELINE_TOML, QianjiApp, QianjiManifest};
use xiuxian_wendao::LinkGraphIndex;

#[test]
fn memory_promotion_manifest_contains_required_nodes_and_binding() {
    let manifest: QianjiManifest = toml::from_str(MEMORY_PROMOTION_PIPELINE_TOML)
        .unwrap_or_else(|error| panic!("memory promotion manifest should parse: {error}"));

    assert!(
        manifest
            .nodes
            .iter()
            .any(|node| node.id == "ReAct_Evidence_Gatherer")
    );
    assert!(
        manifest
            .nodes
            .iter()
            .any(|node| node.id == "Graph_Structure_Validator")
    );
    assert!(manifest.nodes.iter().any(|node| node.id == "Omega_Arbiter"));
    assert!(
        manifest
            .nodes
            .iter()
            .any(|node| node.id == "Wendao_Ingester")
    );

    let annotator = manifest
        .nodes
        .iter()
        .find(|node| node.id == "Promotion_Annotator")
        .unwrap_or_else(|| panic!("Promotion_Annotator node should exist"));
    let binding = annotator
        .qianhuan
        .as_ref()
        .unwrap_or_else(|| panic!("Promotion_Annotator should declare qianhuan binding"));
    assert_eq!(binding.persona_id.as_deref(), Some("artisan-engineer"));
    assert_eq!(
        binding.template_target.as_deref(),
        Some("draft_reflection.md")
    );
}

#[tokio::test]
async fn memory_promotion_pipeline_executes_and_returns_branch_decision() {
    let temp_dir = tempfile::tempdir()
        .unwrap_or_else(|error| panic!("temp dir should be created successfully: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp_dir.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety rules".to_string(),
        None,
    ));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    let scheduler =
        QianjiApp::create_memory_promotion_pipeline(index, orchestrator, registry, None)
            .unwrap_or_else(|error| panic!("memory promotion pipeline should compile: {error}"));
    let result = scheduler
        .run(json!({
            "query": "audit trail traceability architectural consistency for recurring workaround",
            "omega_confidence": 1.0
        }))
        .await
        .unwrap_or_else(|error| panic!("memory promotion pipeline should execute: {error}"));

    let selected_route = result
        .get("selected_route")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("selected_route should exist"));
    let decision = result
        .get("promotion_decision")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("promotion_decision should exist"));

    match selected_route {
        "Promote" => {
            assert_eq!(decision, "promote");
            assert!(
                result["promotion_entity"]
                    .as_object()
                    .is_some_and(|entity| entity.contains_key("id"))
            );
            assert!(
                result["promotion_relation"]
                    .as_object()
                    .is_some_and(|relation| relation.contains_key("id"))
            );
        }
        "Retain" => assert_eq!(decision, "retain"),
        "Obsolete" => assert_eq!(decision, "obsolete"),
        other => panic!("unexpected selected route: {other}"),
    }

    assert_eq!(result["annotated_template_target"], "draft_reflection.md");
}
