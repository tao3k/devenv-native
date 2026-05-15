use std::{error::Error, fs, path::Path};

use crate::studio::arrow_types::{LanceArray, LanceRecordBatch, LanceStringArray};
use xiuxian_wendao_server::transport::{
    SemanticScopeFlightRequest, SemanticScopeFlightRouteProvider,
};

use super::StudioSemanticScopeFlightRouteProvider;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[tokio::test]
async fn semantic_scope_provider_serves_candidate_request_with_full_metadata() -> TestResult {
    let tempdir = tempfile::tempdir()?;
    let semantic_root = tempdir.path().join("semantic");
    write_semantic_scope_fixture(&semantic_root)?;

    let provider = StudioSemanticScopeFlightRouteProvider::from_semantic_root(semantic_root);
    let response = provider
        .semantic_scope_batch(&SemanticScopeFlightRequest {
            task_id: Some("task.semantic-scope-pilot".to_string()),
            object_ids: Vec::new(),
        })
        .await?;

    assert_eq!(response.batch.num_rows(), 2);
    assert_semantic_scope_rows(&response.batch)?;
    assert_semantic_scope_metadata(&serde_json::from_slice(&response.app_metadata)?)?;
    Ok(())
}

fn write_semantic_scope_fixture(semantic_root: &Path) -> std::io::Result<()> {
    create_fixture_dirs(semantic_root)?;
    write_object_fixtures(semantic_root)?;
    write_projection_fixture(semantic_root)?;
    write_change_intent_fixture(semantic_root)
}

fn create_fixture_dirs(semantic_root: &Path) -> std::io::Result<()> {
    fs::create_dir_all(semantic_root.join("objects/component"))?;
    fs::create_dir_all(semantic_root.join("objects/invariant"))?;
    fs::create_dir_all(semantic_root.join("objects/task"))?;
    fs::create_dir_all(semantic_root.join("change-intents"))?;
    fs::create_dir_all(semantic_root.join("projections"))
}

fn write_object_fixtures(semantic_root: &Path) -> std::io::Result<()> {
    fs::write(
        semantic_root.join("objects/component/demo.md"),
        COMPONENT_OBJECT,
    )?;
    fs::write(
        semantic_root.join("objects/invariant/authority.md"),
        INVARIANT_OBJECT,
    )?;
    fs::write(semantic_root.join("objects/task/pilot.md"), TASK_OBJECT)
}

fn write_projection_fixture(semantic_root: &Path) -> std::io::Result<()> {
    fs::write(
        semantic_root.join("projections/llm-compression.md"),
        PROJECTION,
    )
}

fn write_change_intent_fixture(semantic_root: &Path) -> std::io::Result<()> {
    fs::write(
        semantic_root.join("change-intents/semantic-scope-pilot.md"),
        CHANGE_INTENT,
    )
}

fn assert_semantic_scope_rows(batch: &LanceRecordBatch) -> TestResult {
    let object_ids = string_column(batch, "objectId")?;
    assert_eq!(object_ids.value(0), "component.demo");
    assert_eq!(object_ids.value(1), "task.semantic-scope-pilot");

    let change_intents = string_column(batch, "changeIntentIdsJson")?;
    assert_eq!(
        change_intents.value(0),
        r#"["change.semantic-scope-pilot"]"#
    );
    assert_eq!(
        change_intents.value(1),
        r#"["change.semantic-scope-pilot"]"#
    );
    Ok(())
}

fn assert_semantic_scope_metadata(metadata: &serde_json::Value) -> TestResult {
    assert_eq!(
        metadata["semanticScopeBundle"]["projection_revision"],
        "semantic-scope-test"
    );
    assert_eq!(
        metadata["semanticScopeBundle"]["projection_staleness"],
        "stale"
    );
    assert_eq!(array_len(metadata, &["semanticScopeBundle", "objects"])?, 2);
    assert_eq!(
        array_len(metadata, &["semanticScopeBundle", "change_intents"])?,
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
        array_len(metadata, &["semanticSqlGuardEvidence", "findings"])?,
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
        array_len(
            metadata,
            &["semanticProjectionPolicyEvidence", "projections"]
        )?,
        1
    );
    Ok(())
}

fn string_column<'a>(
    batch: &'a LanceRecordBatch,
    column: &str,
) -> TestResult<&'a LanceStringArray> {
    batch
        .column_by_name(column)
        .ok_or_else(|| test_error(format!("missing column `{column}`")))?
        .as_any()
        .downcast_ref::<LanceStringArray>()
        .ok_or_else(|| test_error(format!("column `{column}` should be utf8")))
}

fn array_len(metadata: &serde_json::Value, path: &[&str]) -> TestResult<usize> {
    let mut value = metadata;
    for key in path {
        value = value
            .get(*key)
            .ok_or_else(|| test_error(format!("missing metadata key `{key}`")))?;
    }
    value.as_array().map(Vec::len).ok_or_else(|| {
        test_error(format!(
            "metadata path `{}` should be array",
            path.join(".")
        ))
    })
}

fn test_error(message: String) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message))
}

const COMPONENT_OBJECT: &str = r"---
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
";

const INVARIANT_OBJECT: &str = r"---
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
";

const TASK_OBJECT: &str = r"---
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
";

const PROJECTION: &str = r"---
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
";

const CHANGE_INTENT: &str = r"---
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
";
