//! Integration tests for deterministic docs-facing projected page-index trees.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsPageIndexTreesQuery, docs_page_index_trees_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_page_index_trees_lookup_resolves_deterministic_tree_set() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-page-index-trees.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-page-index-trees]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_page_index_trees_from_config(
        &DocsPageIndexTreesQuery {
            repo_id: "modelica-docs-page-index-trees".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_page_index_trees_modelica_result", json!(result));
    Ok(())
}
