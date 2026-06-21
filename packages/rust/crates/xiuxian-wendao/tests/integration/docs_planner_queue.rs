//! Integration tests for deterministic docs-facing deep-wiki planner queue shaping.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsPlannerQueueQuery, docs_planner_queue_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_planner_queue_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-planner-queue.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-planner-queue]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_planner_queue_from_config(
        &DocsPlannerQueueQuery {
            repo_id: "modelica-docs-planner-queue".to_string(),
            gap_kind: None,
            page_kind: None,
            per_kind_limit: 3,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(
        result.total_gap_count,
        result.groups.iter().map(|group| group.count).sum::<usize>(),
        "planner queue total should match grouped counts"
    );
    assert!(
        result.groups.iter().all(|group| group.gaps.len() <= 3),
        "planner queue previews should honor per-kind truncation"
    );
    assert_repo_json_snapshot("docs_planner_queue_modelica_result", json!(result));
    Ok(())
}
