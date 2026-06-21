//! Integration tests for deterministic docs-facing deep-wiki planner search.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsPlannerSearchQuery, docs_planner_search_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_planner_search_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    fs::write(
        repo_dir.join("Controllers").join("NoDocs.mo"),
        "within Projectionica.Controllers;\nmodel NoDocs\nend NoDocs;\n",
    )?;
    let config_path = temp.path().join("modelica-docs-planner-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-planner-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_planner_search_from_config(
        &DocsPlannerSearchQuery {
            repo_id: "modelica-docs-planner-search".to_string(),
            query: "NoDocs".to_string(),
            gap_kind: None,
            page_kind: None,
            limit: 4,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert!(
        !result.hits.is_empty(),
        "planner search should return at least one deterministic hit"
    );
    assert!(
        result.hits.len() <= 4,
        "planner search should honor the configured hit limit"
    );
    assert!(
        result
            .hits
            .iter()
            .all(|hit| hit.gap.title.contains("NoDocs") || hit.gap.page_id.contains("NoDocs")),
        "planner search hits should stay anchored to the injected no-doc target"
    );
    assert!(
        result.hits.windows(2).all(|window| {
            let left = &window[0];
            let right = &window[1];
            (
                std::cmp::Reverse(left.search_score),
                left.gap.kind,
                left.gap.title.as_str(),
                left.gap.gap_id.as_str(),
            ) <= (
                std::cmp::Reverse(right.search_score),
                right.gap.kind,
                right.gap.title.as_str(),
                right.gap.gap_id.as_str(),
            )
        }),
        "planner search hits should stay in deterministic score order"
    );

    assert_repo_json_snapshot("docs_planner_search_modelica_result", json!(result));
    Ok(())
}
