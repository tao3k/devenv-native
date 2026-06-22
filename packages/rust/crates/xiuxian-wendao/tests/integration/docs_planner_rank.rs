//! Integration tests for deterministic docs-facing deep-wiki planner ranking.
#![cfg(feature = "julia")]

use std::cmp::Reverse;
use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsPlannerRankQuery, docs_planner_rank_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_planner_rank_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-planner-rank.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-planner-rank]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_planner_rank_from_config(
        &DocsPlannerRankQuery {
            repo_id: "modelica-docs-planner-rank".to_string(),
            gap_kind: None,
            page_kind: None,
            limit: 4,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert!(
        result.hits.len() <= 4,
        "planner rank should honor the configured hit limit"
    );
    assert!(
        result.hits.iter().all(|hit| !hit.reasons.is_empty()),
        "planner rank should keep deterministic score explanations"
    );
    assert!(
        result.hits.windows(2).all(|window| {
            let left = &window[0];
            let right = &window[1];
            (
                Reverse(left.priority_score),
                left.gap.kind,
                left.gap.title.as_str(),
                left.gap.gap_id.as_str(),
            ) <= (
                Reverse(right.priority_score),
                right.gap.kind,
                right.gap.title.as_str(),
                right.gap.gap_id.as_str(),
            )
        }),
        "planner rank hits should stay in deterministic priority order"
    );

    assert_repo_json_snapshot("docs_planner_rank_modelica_result", json!(result));
    Ok(())
}
