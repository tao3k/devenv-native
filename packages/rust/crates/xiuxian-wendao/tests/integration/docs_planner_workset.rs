//! Integration tests for deterministic docs-facing deep-wiki planner workset shaping.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPlannerWorksetQuery, ProjectionPageKind, docs_planner_workset_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_planner_workset_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-planner-workset.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-planner-workset]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_planner_workset_from_config(
        &DocsPlannerWorksetQuery {
            repo_id: "modelica-docs-planner-workset".to_string(),
            gap_kind: None,
            page_kind: None,
            per_kind_limit: 3,
            limit: 4,
            family_kind: Some(ProjectionPageKind::HowTo),
            related_limit: 3,
            family_limit: 3,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(
        result.queue.total_gap_count,
        result
            .queue
            .groups
            .iter()
            .map(|group| group.count)
            .sum::<usize>(),
        "planner workset queue total should match grouped counts"
    );
    assert_eq!(
        result.items.len(),
        result.ranked_hits.len(),
        "planner workset should reopen every ranked hit into one item"
    );
    assert!(
        result.items.len() <= 4,
        "planner workset should honor the ranked-hit limit"
    );
    assert_eq!(
        result
            .groups
            .iter()
            .map(|group| group.selected_count)
            .sum::<usize>(),
        result.items.len(),
        "planner workset grouped counts should match opened items"
    );
    assert!(
        result.groups.iter().all(|group| {
            group.selected_count == group.ranked_hits.len()
                && group.selected_count == group.items.len()
                && group.families.iter().all(|family| {
                    family.selected_count == family.ranked_hits.len()
                        && family.selected_count == family.items.len()
                })
        }),
        "planner workset groups should stay internally aligned"
    );

    assert_repo_json_snapshot("docs_planner_workset_modelica_result", json!(result));
    Ok(())
}
