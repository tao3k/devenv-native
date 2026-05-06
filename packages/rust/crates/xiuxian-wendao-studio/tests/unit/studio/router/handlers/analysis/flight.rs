use std::fs;

use crate::studio::arrow_types::{LanceArray, LanceStringArray};
use xiuxian_wendao_server::transport::{
    SemanticScopeFlightRequest, SemanticScopeFlightRouteProvider,
};

use super::StudioSemanticScopeFlightRouteProvider;

#[tokio::test]
async fn semantic_scope_provider_serves_candidate_request_with_full_metadata() {
    let tempdir = tempfile::tempdir().expect("create temp semantic root");
    let semantic_root = tempdir.path().join("semantic");
    fs::create_dir_all(semantic_root.join("objects/component"))
        .expect("create component object directory");
    fs::create_dir_all(semantic_root.join("objects/invariant"))
        .expect("create invariant object directory");
    fs::create_dir_all(semantic_root.join("objects/task")).expect("create task directory");
    fs::create_dir_all(semantic_root.join("change-intents"))
        .expect("create change intent directory");
    fs::create_dir_all(semantic_root.join("projections")).expect("create projections");
    fs::write(
        semantic_root.join("objects/component/demo.md"),
        r"---
id: component.demo
kind: component
title: Demo Component
status: active
confidence:
  score: 0.92
  source: verified
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
role: runtime-provider
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
- cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Component
",
    )
    .expect("write component object");
    fs::write(
        semantic_root.join("objects/invariant/authority.md"),
        r"---
id: invariant.demo-authority
kind: invariant
title: Demo Authority Invariant
status: active
confidence:
  score: 0.95
  source: verified
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
role: runtime-provider
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
- cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations: []
---

# Demo Authority Invariant
",
    )
    .expect("write invariant object");
    fs::write(
        semantic_root.join("objects/task/pilot.md"),
        r"---
id: task.semantic-scope-pilot
kind: task
title: Semantic Scope Pilot
status: candidate
confidence:
  score: 0.8
  source: llm_suggested
owners:
  - scope: packages/rust/crates/xiuxian-wendao-studio
role: route-adapter
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
- cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
relations:
  - kind: implements
target: component.demo
---

# Semantic Scope Pilot
",
    )
    .expect("write task object");
    fs::write(
        semantic_root.join("projections/llm-compression.md"),
        r"---
type: semantic_projection
projection: llm_compression
source_objects:
  - component.demo
  - task.semantic-scope-pilot
source_revision: stale-fixture
projection_revision: semantic-scope-test
staleness: stale
status: active
---

# LLM Compression
",
    )
    .expect("write projection");
    fs::write(
        semantic_root.join("change-intents/semantic-scope-pilot.md"),
        r"---
type: semantic_change_intent
id: change.semantic-scope-pilot
title: Semantic Scope Pilot
status: active
touched_objects:
  - task.semantic-scope-pilot
changed_relations:
  - source: task.semantic-scope-pilot
kind: implements
target: component.demo
action: add
affected_invariants:
  - invariant.demo-authority
required_validations:
  - cargo test -p xiuxian-wendao-studio --features zhenfa-router --test semantic_scope_provider semantic_scope
projections_to_refresh:
  - llm_compression
candidate_suggestions:
  - task.semantic-scope-pilot
---

# Semantic Scope Pilot Change
",
    )
    .expect("write change intent");

    let provider = StudioSemanticScopeFlightRouteProvider::from_semantic_root(semantic_root);
    let response = provider
        .semantic_scope_batch(&SemanticScopeFlightRequest {
            task_id: Some("task.semantic-scope-pilot".to_string()),
            object_ids: Vec::new(),
        })
        .await
        .expect("semantic scope response");

    assert_eq!(response.batch.num_rows(), 2);
    let object_ids = response
        .batch
        .column_by_name("objectId")
        .expect("objectId column")
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .expect("objectId string column");
    assert_eq!(object_ids.value(0), "component.demo");
    assert_eq!(object_ids.value(1), "task.semantic-scope-pilot");
    let change_intents = response
        .batch
        .column_by_name("changeIntentIdsJson")
        .expect("changeIntentIdsJson column")
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .expect("changeIntentIdsJson string column");
    assert_eq!(
        change_intents.value(0),
        r#"["change.semantic-scope-pilot"]"#
    );
    assert_eq!(
        change_intents.value(1),
        r#"["change.semantic-scope-pilot"]"#
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&response.app_metadata).expect("semantic metadata json");
    assert_eq!(
        metadata["semanticScopeBundle"]["projection_revision"],
        "semantic-scope-test"
    );
    assert_eq!(
        metadata["semanticScopeBundle"]["projection_staleness"],
        "stale"
    );
    assert_eq!(
        metadata["semanticScopeBundle"]["objects"]
            .as_array()
            .expect("objects metadata")
            .len(),
        2
    );
    assert_eq!(
        metadata["semanticScopeBundle"]["change_intents"]
            .as_array()
            .expect("change intent metadata")
            .len(),
        1
    );
    assert_eq!(
        metadata["semanticSqlGuardEvidence"]["guardId"],
        "semantic_sql.projection_freshness"
    );
    assert_eq!(
        metadata["semanticSqlGuardEvidence"]["status"],
        "review_required"
    );
    assert_eq!(metadata["semanticSqlGuardEvidence"]["failingRowCount"], 1);
    assert_eq!(
        metadata["semanticSqlGuardEvidence"]["findings"]
            .as_array()
            .expect("semantic SQL guard findings metadata")
            .len(),
        1
    );
    assert_eq!(
        metadata["semanticProjectionPolicyEvidence"]["policyId"],
        "semantic_projection.required_refresh_targets"
    );
    assert_eq!(
        metadata["semanticProjectionPolicyEvidence"]["status"],
        "review_required"
    );
    assert_eq!(
        metadata["semanticProjectionPolicyEvidence"]["failingProjectionCount"],
        1
    );
    assert_eq!(
        metadata["semanticProjectionPolicyEvidence"]["projections"]
            .as_array()
            .expect("semantic projection policy finding metadata")
            .len(),
        1
    );
}
