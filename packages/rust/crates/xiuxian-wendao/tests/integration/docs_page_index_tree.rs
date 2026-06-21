//! Integration tests for deterministic docs-facing projected page-index tree lookup.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPageIndexTreeQuery, RepoProjectedPageIndexTreesQuery, docs_page_index_tree_from_config,
    repo_projected_page_index_trees_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_page_index_tree_lookup_resolves_one_stable_tree() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-page-index-tree.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-page-index-tree]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-docs-page-index-tree".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(page_id) = trees
        .trees
        .iter()
        .find(|tree| tree.title == "Projectionica.Controllers.PI")
        .map(|tree| tree.page_id.clone())
    else {
        panic!("expected a projected page-index tree titled `Projectionica.Controllers.PI`");
    };

    let result = docs_page_index_tree_from_config(
        &DocsPageIndexTreeQuery {
            repo_id: "modelica-docs-page-index-tree".to_string(),
            page_id,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_page_index_tree_modelica_result", json!(result));
    Ok(())
}
