//! Integration tests for deterministic docs-facing projected page-index tree search.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPageIndexTreeSearchQuery, ProjectionPageKind, docs_page_index_tree_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_page_index_tree_search_matches_section_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-page-index-tree-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-page-index-tree-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_page_index_tree_search_from_config(
        &DocsPageIndexTreeSearchQuery {
            repo_id: "modelica-docs-page-index-tree-search".to_string(),
            query: "anchors".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_page_index_tree_search_modelica_result", json!(result));
    Ok(())
}
