//! Integration tests for deterministic projected page navigation bundles.

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_projection_support::{assert_repo_json_snapshot, write_repo_config};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    ProjectedPageIndexNode, ProjectionPageKind, RepoProjectedPageIndexTreesQuery,
    RepoProjectedPageNavigationQuery, RepoProjectedPagesQuery,
    repo_projected_page_index_trees_from_config, repo_projected_page_navigation_from_config,
    repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_navigation_bundle_resolves_tree_context_and_family_cluster() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_navigation_julia_repo(temp.path(), "ProjectionPkg")?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "projection-sample")?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "projection-sample".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let page = pages
        .pages
        .iter()
        .find(|page| {
            page.kind == ProjectionPageKind::Reference
                && page.title == "ProjectionPkg.solve"
                && page.page_id.contains(":symbol:")
        })
        .unwrap_or_else(|| {
            panic!("expected a symbol-backed projected reference page titled `ProjectionPkg.solve`")
        });

    let trees = repo_projected_page_index_trees_from_config(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: "projection-sample".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let tree = trees
        .trees
        .iter()
        .find(|tree| tree.page_id == page.page_id)
        .unwrap_or_else(|| panic!("expected a projected page-index tree for the selected page"));
    let node_id = find_node_id(tree.roots.as_slice(), "Anchors")
        .unwrap_or_else(|| panic!("expected a projected page-index node titled `Anchors`"));

    let result = repo_projected_page_navigation_from_config(
        &RepoProjectedPageNavigationQuery {
            repo_id: "projection-sample".to_string(),
            page_id: page.page_id.clone(),
            node_id: Some(node_id),
            family_kind: Some(ProjectionPageKind::HowTo),
            related_limit: 3,
            family_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_projected_page_navigation_result", json!(result));
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_navigation_bundle_resolves_tree_context() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-projected-navigation.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-projected-navigation]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-projected-navigation".to_string(),
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
            repo_id: "modelica-projected-navigation".to_string(),
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

    let result = repo_projected_page_navigation_from_config(
        &RepoProjectedPageNavigationQuery {
            repo_id: "modelica-projected-navigation".to_string(),
            page_id: page.page_id.clone(),
            node_id: Some(node_id),
            family_kind: None,
            related_limit: 3,
            family_limit: 0,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_navigation_modelica_result",
        json!(result),
    );
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

fn create_navigation_julia_repo(
    base: &Path,
    package_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repo_dir = base.join(package_name.to_ascii_lowercase());
    fs::create_dir_all(repo_dir.join("src"))?;
    fs::write(
        repo_dir.join("Project.toml"),
        format!(
            r#"name = "{package_name}"
uuid = "12345678-1234-1234-1234-123456789abc"
version = "0.1.0"
"#
        ),
    )?;
    fs::write(repo_dir.join("README.md"), "# Projection Repo\n")?;
    fs::write(
        repo_dir.join("src").join(format!("{package_name}.jl")),
        format!(
            "module {package_name}\nexport solve\n\"\"\"solve docs\"\"\"\nsolve() = nothing\nend\n"
        ),
    )?;
    fs::create_dir_all(repo_dir.join("examples"))?;
    fs::write(
        repo_dir.join("examples").join("solve_demo.jl"),
        format!("using {package_name}\nsolve()\n"),
    )?;
    fs::create_dir_all(repo_dir.join("docs"))?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    initialize_git_repository(
        repo_dir.as_path(),
        &format!(
            "https://example.invalid/{}/{}.git",
            "xiuxian-wendao",
            package_name.to_ascii_lowercase()
        ),
    )?;
    Ok(repo_dir)
}

fn initialize_git_repository(
    repo_dir: &Path,
    remote_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::support::repo_fixture::initialize_git_repository(repo_dir, remote_url)
}
