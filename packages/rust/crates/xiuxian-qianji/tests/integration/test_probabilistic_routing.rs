#![cfg(feature = "wendao-integration")]

use serde_json::{Value, json};
use std::sync::Arc;
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

const BRANCH_TOML: &str = include_str!("../../resources/tests/probabilistic_branch.toml");
const SEMANTIC_GUARD_ROUTE_BRANCH_TOML: &str =
    include_str!("../../resources/tests/semantic_guard_route_branch.toml");

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_probabilistic_routing_from_resource() -> TestResult {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);

    let compiler = QianjiCompiler::new(index);
    let engine = compiler.compile(BRANCH_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler.run(json!({})).await?;

    assert_eq!(result["selected_route"], "PathA");
    assert_eq!(result["BranchA"], "done");
    Ok(())
}

#[tokio::test]
async fn semantic_guard_route_fixture_routes_review_branch() -> TestResult {
    let temp = tempfile::tempdir()?;
    let index = Arc::new(LinkGraphIndex::build(temp.path())?);

    let compiler = QianjiCompiler::new(index);
    let engine = compiler.compile(SEMANTIC_GUARD_ROUTE_BRANCH_TOML)?;
    let scheduler = QianjiScheduler::new(engine);

    let result = scheduler
        .run(json!({
            "semanticScopeGuardPolicy": "block_on_blocked",
            "semanticScopeMetadata": semantic_scope_metadata_value("stale", &[]),
            "omega_confidence": -1.0
        }))
        .await?;

    assert_eq!(result["selected_route"], "review_required");
    assert_eq!(result["ReviewPath"], "done");
    assert!(result.get("ContinuePath").is_none());
    assert!(result.get("BlockedPath").is_none());
    Ok(())
}

fn semantic_scope_metadata_value(projection_staleness: &str, unresolved_ids: &[&str]) -> Value {
    json!({
        "semanticScopeBundle": {
            "task_id": "task.demo",
            "requested_object_ids": ["task.demo"],
            "objects": [
                {
                    "id": "task.demo",
                    "kind": "task",
                    "title": "Demo Task",
                    "status": "active",
                    "confidence": {
                        "score": 0.95,
                        "source": "verified"
                    },
                    "owners": [
                        {
                            "scope": "xiuxian-qianji",
                            "role": "workflow_semantic_scope_consumer"
                        }
                    ],
                    "provenance": {
                        "source": "semantic/objects/task/demo.md",
                        "recorded_by": "test",
                        "recorded_at": "2026-05-05"
                    },
                    "verification": {
                        "required": ["cargo test -p xiuxian-qianji semantic_guard_route"]
                    },
                    "relations": []
                }
            ],
            "relations": [],
            "change_intents": [
                {
                    "type": "semantic_change_intent",
                    "id": "change.demo",
                    "title": "Demo Change",
                    "status": "active",
                    "touched_objects": ["task.demo"],
                    "affected_invariants": [],
                    "required_validations": ["cargo test -p xiuxian-qianji semantic_guard_route"],
                    "projections_to_refresh": ["llm_compression"]
                }
            ],
            "affected_invariants": [],
            "required_validations": ["cargo test -p xiuxian-qianji semantic_guard_route"],
            "projection_revision": "semantic-guard-route-fixture-demo",
            "projection_source_revision": "blake3:demo",
            "projection_staleness": projection_staleness,
            "provenance": [
                {
                    "object_id": "task.demo",
                    "source_path": "semantic/objects/task/demo.md",
                    "source": "semantic/objects/task/demo.md"
                }
            ],
            "unresolved_ids": unresolved_ids
        }
    })
}
