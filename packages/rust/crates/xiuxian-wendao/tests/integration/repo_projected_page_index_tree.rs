//! Integration tests for deterministic projected page-index tree lookup.

#[cfg(feature = "julia")]
use std::fs;

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{assert_repo_json_snapshot, sample_projection_analysis};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreesQuery,
    build_repo_projected_page_index_tree, build_repo_projected_page_index_trees,
};
#[cfg(feature = "julia")]
use xiuxian_wendao::analyzers::{
    repo_projected_page_index_tree_from_config, repo_projected_page_index_trees_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_index_tree_lookup_resolves_one_stable_tree() -> TestResult {
    let analysis = sample_projection_analysis("projection-sample");

    let trees = build_repo_projected_page_index_trees(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "projection-sample".to_string(),
        },
        &analysis,
    )?;

    let Some(page_id) = trees
        .trees
        .iter()
        .find(|tree| tree.title == "solve")
        .map(|tree| tree.page_id.clone())
    else {
        panic!("expected a projected page-index tree titled `solve`");
    };

    let result = build_repo_projected_page_index_tree(
        &RepoProjectedPageIndexTreeQuery {
            repo_id: "projection-sample".to_string(),
            page_id,
        },
        &analysis,
    )?;

    assert_repo_json_snapshot("repo_projected_page_index_tree_result", json!(result));
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_index_tree_lookup_resolves_one_stable_tree() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-projected-index-tree.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-projected-index-tree]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "modelica-projected-index-tree".to_string(),
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

    let result = repo_projected_page_index_tree_from_config(
        &RepoProjectedPageIndexTreeQuery {
            repo_id: "modelica-projected-index-tree".to_string(),
            page_id,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_index_tree_modelica_result",
        json!(result),
    );
    Ok(())
}
