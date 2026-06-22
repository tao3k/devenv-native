//! Integration tests for deterministic projected page-family cluster lookup.

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_projection_support::{assert_repo_json_snapshot, write_repo_config};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    ProjectionPageKind, RepoProjectedPageFamilyClusterQuery, RepoProjectedPagesQuery,
    repo_projected_page_family_cluster_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_family_cluster_lookup_resolves_how_to_cluster_for_reference_page() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_gateway_style_julia_repo(temp.path(), "ProjectionPkg")?;
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
                && page.title == "solve"
                && page.page_id.contains(":doc:")
        })
        .unwrap_or_else(|| panic!("expected a doc-backed projected reference page titled `solve`"));

    let result = repo_projected_page_family_cluster_from_config(
        &RepoProjectedPageFamilyClusterQuery {
            repo_id: "projection-sample".to_string(),
            page_id: page.page_id.clone(),
            kind: ProjectionPageKind::HowTo,
            limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_projected_page_family_cluster_result", json!(result));
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_family_cluster_resolves_how_to_cluster_for_reference_page()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-family-cluster.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-family-cluster]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-family-cluster".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(page) = pages.pages.iter().find(|page| {
        page.kind == ProjectionPageKind::Reference && page.title == "Projectionica.Controllers"
    }) else {
        panic!(
            "expected a module-backed projected reference page titled `Projectionica.Controllers`"
        );
    };

    let result = repo_projected_page_family_cluster_from_config(
        &RepoProjectedPageFamilyClusterQuery {
            repo_id: "modelica-family-cluster".to_string(),
            page_id: page.page_id.clone(),
            kind: ProjectionPageKind::HowTo,
            limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_family_cluster_modelica_result",
        json!(result),
    );
    Ok(())
}

fn create_gateway_style_julia_repo(
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
