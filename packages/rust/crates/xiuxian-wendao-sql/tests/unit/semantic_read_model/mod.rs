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
    semantic_read_model_catalog, semantic_read_model_materialization_plan,
    semantic_read_model_materialization_preflight, semantic_read_model_snapshot,
    semantic_read_model_snapshot_check, validate_semantic_read_model_query_text,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

mod catalog;
mod guard;
mod materialization;
mod query;
mod snapshot;

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
