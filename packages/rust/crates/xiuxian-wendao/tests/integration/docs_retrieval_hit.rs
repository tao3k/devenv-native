//! Integration tests for deterministic docs-facing mixed projected retrieval hit lookup.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsRetrievalHitQuery, ProjectedPageIndexNode, ProjectionPageKind,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPagesQuery, docs_retrieval_hit_from_config,
    repo_projected_page_index_trees_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_retrieval_hit_resolves_node_hit() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-retrieval-hit.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-retrieval-hit]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-docs-retrieval-hit".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(page) = pages.pages.iter().find(|page| {
        page.kind == ProjectionPageKind::Reference
            && page.title == "Projectionica.Controllers.PI"
            && page.page_id.contains(":symbol:")
    }) else {
        panic!(
            "expected a symbol-backed projected reference page titled `Projectionica.Controllers.PI`"
        );
    };

    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-docs-retrieval-hit".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(tree) = trees.trees.iter().find(|tree| tree.page_id == page.page_id) else {
        panic!("expected a projected page-index tree for the selected page");
    };
    let Some(node_id) = find_node_id(tree.roots.as_slice(), "Anchors") else {
        panic!("expected a projected page-index node titled `Anchors`");
    };

    let result = docs_retrieval_hit_from_config(
        &DocsRetrievalHitQuery {
            repo: "modelica-docs-retrieval-hit".to_string(),
            page: page.page_id.clone(),
            node: Some(node_id),
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_retrieval_hit_modelica_result", json!(result));
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
