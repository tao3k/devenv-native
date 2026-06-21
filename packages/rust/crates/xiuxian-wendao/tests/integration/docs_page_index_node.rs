//! Integration tests for deterministic docs-facing projected page-index node lookup.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPageIndexNodeQuery, ProjectedPageIndexNode, RepoProjectedPageIndexTreesQuery,
    docs_page_index_node_from_config, repo_projected_page_index_trees_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_page_index_node_lookup_resolves_one_stable_node() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-page-index-node.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-page-index-node]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-docs-page-index-node".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(tree) = trees
        .trees
        .iter()
        .find(|tree| tree.title == "Projectionica.Controllers.PI")
    else {
        panic!("expected a projected page-index tree titled `Projectionica.Controllers.PI`");
    };
    let Some(node_id) = find_node_id(tree.roots.as_slice(), "Anchors") else {
        panic!("expected a projected page-index node titled `Anchors`");
    };

    let result = docs_page_index_node_from_config(
        &DocsPageIndexNodeQuery {
            repo_id: "modelica-docs-page-index-node".to_string(),
            page_id: tree.page_id.clone(),
            node_id,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_page_index_node_modelica_result", json!(result));
    Ok(())
}

fn find_node_id(nodes: &[ProjectedPageIndexNode], title: &str) -> Option<String> {
    for node in nodes {
        if node.title == title {
            return Some(node.node_id.clone());
        }
        if let Some(node_id) = find_node_id(node.children.as_slice(), title) {
            return Some(node_id);
        }
    }
    None
}
