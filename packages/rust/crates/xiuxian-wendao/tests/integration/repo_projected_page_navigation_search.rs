//! Integration tests for deterministic projected page navigation search.

#[cfg(feature = "julia")]
use std::fs;

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{assert_repo_json_snapshot, sample_projection_analysis};
use serde_json::json;
#[cfg(feature = "julia")]
use xiuxian_wendao::analyzers::repo_projected_page_navigation_search_from_config;
use xiuxian_wendao::analyzers::{
    ProjectionPageKind, RepoProjectedPageNavigationSearchQuery,
    build_repo_projected_page_navigation_search,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_navigation_search_expands_reference_hits_into_navigation_bundles() -> TestResult {
    let analysis = sample_projection_analysis("projection-sample");
    let result = build_repo_projected_page_navigation_search(
        &RepoProjectedPageNavigationSearchQuery {
            repo_id: "projection-sample".to_string(),
            query: "solve".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            family_kind: None,
            limit: 3,
            related_limit: 3,
            family_limit: 2,
        },
        &analysis,
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_navigation_search_result",
        json!(result),
    );
    Ok(())
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_navigation_search_expands_reference_hits_into_navigation_bundles()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-navigation-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-navigation-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = repo_projected_page_navigation_search_from_config(
        &RepoProjectedPageNavigationSearchQuery {
            repo_id: "modelica-navigation-search".to_string(),
            query: "Projectionica.Controllers".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            family_kind: Some(ProjectionPageKind::HowTo),
            limit: 2,
            related_limit: 3,
            family_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_navigation_search_modelica_result",
        json!(result),
    );
    Ok(())
}
