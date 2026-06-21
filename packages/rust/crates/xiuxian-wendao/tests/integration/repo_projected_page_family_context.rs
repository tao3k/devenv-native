//! Integration tests for deterministic projected page-family context.

#[cfg(feature = "julia")]
use std::fs;

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    ProjectionPageKind, RepoProjectedPageFamilyContextQuery, RepoProjectedPagesQuery,
    repo_projected_page_family_context_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_family_context_lookup_groups_related_pages_by_family() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ProjectionPkg", true)?;
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
        .find(|page| page.kind == ProjectionPageKind::HowTo)
        .unwrap_or_else(|| panic!("expected a projected how-to page"));

    let result = repo_projected_page_family_context_from_config(
        &RepoProjectedPageFamilyContextQuery {
            repo_id: "projection-sample".to_string(),
            page_id: page.page_id.clone(),
            per_kind_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_projected_page_family_context_result", json!(result));
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_family_context_groups_related_pages_by_family() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-family-context.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-family-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-family-context".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(page) = pages
        .pages
        .iter()
        .find(|page| page.kind == ProjectionPageKind::HowTo)
    else {
        panic!("expected a projected how-to page");
    };

    let result = repo_projected_page_family_context_from_config(
        &RepoProjectedPageFamilyContextQuery {
            repo_id: "modelica-family-context".to_string(),
            page_id: page.page_id.clone(),
            per_kind_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_family_context_modelica_result",
        json!(result),
    );
    Ok(())
}
