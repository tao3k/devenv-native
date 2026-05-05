use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use tempfile::tempdir;
use xiuxian_wendao_parsers::semantic_ssot::{
    load_semantic_repository, semantic_projection_source_revision,
};

use crate::semantic_read_model::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, SemanticSqlGuardStatus, build_semantic_read_model_rows,
    query_semantic_read_model_payload, run_semantic_sql_projection_freshness_guard,
    validate_semantic_read_model_query_text,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn write_file(path: &Path, body: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)
}

fn write_semantic_read_model_fixture(root: &Path) -> std::io::Result<()> {
    write_file(
        &root.join("objects/component/demo.md"),
        r"---
id: component.demo
kind: component
title: Demo Component
status: active
confidence:
  score: 0.95
  source: verified
owners:
  - scope: xiuxian-wendao-sql
    role: read_model_source
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-sql semantic_read_model
relations:
  - kind: validates
    target: task.demo
---

# Demo Component
",
    )?;
    write_file(
        &root.join("objects/task/demo.md"),
        r"---
id: task.demo
kind: task
title: Demo Task
status: active
confidence:
  score: 0.9
  source: human_signed
owners:
  - scope: xiuxian-wendao-sql
    role: read_model_target
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-sql semantic_read_model
relations: []
---

# Demo Task
",
    )?;
    write_file(
        &root.join("projections/llm-compression.md"),
        r"---
type: semantic_projection
projection: llm_compression
source_objects:
  - component.demo
  - task.demo
source_revision: stale-demo
projection_revision: semantic-read-model-demo
staleness: stale
status: active
---

# LLM Compression
",
    )
}

fn write_invalid_semantic_fixture(root: &Path) -> std::io::Result<()> {
    write_file(
        &root.join("objects/component/broken.md"),
        r"---
id: component.broken
kind: component
title: Broken Component
status: active
confidence:
  score: 0.8
  source: verified
owners:
  - scope: xiuxian-wendao-sql
    role: read_model_source
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test
relations:
  - kind: validates
    target: task.missing
---

# Broken Component
",
    )
}

fn write_semantic_no_projection_fixture(root: &Path) -> std::io::Result<()> {
    write_file(
        &root.join("objects/task/no-stale.md"),
        r"---
id: task.no-stale
kind: task
title: No Stale Projection Task
status: active
confidence:
  score: 0.9
  source: human_signed
owners:
  - scope: xiuxian-wendao-sql
    role: sql_guard_source
provenance:
  source: docs/rfcs/demo.md
  recorded_by: test
  recorded_at: 2026-05-05
verification:
  required:
    - cargo test -p xiuxian-wendao-sql semantic_sql_guard
relations: []
---

# No Stale Projection Task
",
    )
}

fn write_semantic_fresh_projection_fixture(root: &Path) -> std::io::Result<()> {
    write_semantic_no_projection_fixture(root)?;
    write_file(
        &root.join("projections/llm-compression.md"),
        &semantic_projection_fixture(&["task.no-stale"], "stale-demo", "stale"),
    )?;

    let repository = load_semantic_repository(root);
    let projection = repository
        .projections
        .first()
        .ok_or_else(|| std::io::Error::other("projection fixture should load"))?;
    let source_revision = semantic_projection_source_revision(&repository, projection)
        .ok_or_else(|| std::io::Error::other("projection source revision should compute"))?;
    write_file(
        &root.join("projections/llm-compression.md"),
        &semantic_projection_fixture(&["task.no-stale"], &source_revision, "fresh"),
    )
}

fn semantic_projection_fixture(
    source_objects: &[&str],
    source_revision: &str,
    staleness: &str,
) -> String {
    let mut rendered_source_objects = String::new();
    for object_id in source_objects {
        writeln!(&mut rendered_source_objects, "  - {object_id}")
            .unwrap_or_else(|error| panic!("render projection source object: {error}"));
    }
    format!(
        concat!(
            "---\n",
            "type: semantic_projection\n",
            "projection: llm_compression\n",
            "source_objects:\n",
            "{rendered_source_objects}",
            "source_revision: {source_revision}\n",
            "projection_revision: semantic-sql-guard-demo\n",
            "staleness: {staleness}\n",
            "status: active\n",
            "---\n",
            "\n",
            "# LLM Compression\n",
        ),
        rendered_source_objects = rendered_source_objects,
        source_revision = source_revision,
        staleness = staleness,
    )
}

#[test]
fn semantic_read_model_projects_objects_relations_and_projection_state() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let repository = load_semantic_repository(root);
    assert!(
        repository.report.is_success(),
        "semantic fixture should validate: {:?}",
        repository.report.issues
    );
    let rows = build_semantic_read_model_rows(&repository).map_err(std::io::Error::other)?;

    assert_eq!(rows.objects.len(), 2);
    assert_eq!(rows.relations.len(), 1);
    assert_eq!(rows.projection_state.len(), 1);
    assert!(rows.objects.iter().any(|row| {
        row.id == "component.demo"
            && row.read_model_projection_revision == "semantic-read-model-demo"
            && row.read_model_projection_staleness == "stale"
    }));
    assert!(
        rows.relations
            .iter()
            .any(|row| row.source == "component.demo"
                && row.kind == "validates"
                && row.target == "task.demo")
    );
    Ok(())
}

#[tokio::test]
async fn semantic_read_model_query_exposes_registered_tables() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let payload = query_semantic_read_model_payload(
        root,
        "select o.id, r.kind, p.projection_revision, p.staleness \
         from semantic_objects o \
         join semantic_relations r on o.id = r.source \
         cross join semantic_projection_state p \
         order by o.id",
    )
    .await
    .map_err(std::io::Error::other)?;

    assert_eq!(
        payload.metadata.registered_tables,
        vec![
            SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
            SEMANTIC_RELATIONS_TABLE_NAME.to_string(),
            SEMANTIC_PROJECTION_STATE_TABLE_NAME.to_string(),
        ]
    );
    assert_eq!(payload.metadata.registered_table_count, 3);
    assert_eq!(
        payload.metadata.local_relation_engine.as_deref(),
        Some("datafusion")
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("id").and_then(serde_json::Value::as_str) == Some("component.demo")
                    && row.get("kind").and_then(serde_json::Value::as_str) == Some("validates")
                    && row
                        .get("projection_revision")
                        .and_then(serde_json::Value::as_str)
                        == Some("semantic-read-model-demo")
                    && row.get("staleness").and_then(serde_json::Value::as_str) == Some("stale")
            )
    );
    Ok(())
}

#[test]
fn semantic_read_model_rejects_invalid_repository() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_invalid_semantic_fixture(root)?;

    let repository = load_semantic_repository(root);
    assert!(!repository.report.is_success());

    let Err(error) = build_semantic_read_model_rows(&repository) else {
        panic!("invalid semantic repository should not project");
    };
    assert!(error.contains("semantic repository validation failed"));
    Ok(())
}

#[test]
fn semantic_read_model_query_validation_accepts_only_single_read_only_query() {
    assert!(
        validate_semantic_read_model_query_text("select id from semantic_objects").is_ok(),
        "plain select should be accepted"
    );
    assert!(
        validate_semantic_read_model_query_text("").is_err(),
        "blank query should be rejected"
    );
    assert!(
        validate_semantic_read_model_query_text("select id from semantic_objects; select 1")
            .is_err(),
        "multi-statement query should be rejected"
    );
    assert!(
        validate_semantic_read_model_query_text(
            "insert into semantic_objects (id) values ('component.bad')"
        )
        .is_err(),
        "mutation statement should be rejected"
    );
}

#[tokio::test]
async fn semantic_sql_guard_projection_freshness_reports_stale_projection() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_read_model_fixture(root)?;

    let evidence = run_semantic_sql_projection_freshness_guard(root)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(evidence.status, SemanticSqlGuardStatus::ReviewRequired);
    assert_eq!(evidence.status.as_str(), "review_required");
    assert_eq!(evidence.failing_row_count, 1);
    assert_eq!(evidence.findings.len(), 1);
    assert_eq!(evidence.findings[0].projection, "llm_compression");
    assert_eq!(evidence.findings[0].staleness, "stale");
    assert_eq!(
        evidence.local_relation_engine.as_deref(),
        Some("datafusion")
    );
    assert!(evidence.query_text.contains("semantic_projection_state"));
    assert!(evidence.message.contains("requires review"));
    Ok(())
}

#[tokio::test]
async fn semantic_sql_guard_projection_freshness_passes_with_fresh_projection_rows() -> TestResult {
    let temp_dir = tempdir()?;
    let root = temp_dir.path();
    write_semantic_fresh_projection_fixture(root)?;

    let evidence = run_semantic_sql_projection_freshness_guard(root)
        .await
        .map_err(std::io::Error::other)?;

    assert_eq!(evidence.status, SemanticSqlGuardStatus::Passed);
    assert_eq!(evidence.status.as_str(), "passed");
    assert_eq!(evidence.failing_row_count, 0);
    assert!(evidence.findings.is_empty());
    assert!(evidence.message.contains("passed"));
    Ok(())
}
