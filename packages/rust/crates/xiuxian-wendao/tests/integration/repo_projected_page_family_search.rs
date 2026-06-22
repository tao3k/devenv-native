//! Integration tests for deterministic projected page-family search.

#[cfg(feature = "julia")]
use std::fs;

#[cfg(feature = "julia")]
use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{assert_repo_json_snapshot, sample_projection_analysis};
use serde_json::json;
#[cfg(feature = "julia")]
use xiuxian_wendao::analyzers::repo_projected_page_family_search_from_config;
use xiuxian_wendao::analyzers::{
    ProjectionPageKind, RepoProjectedPageFamilySearchQuery, build_repo_projected_page_family_search,
};

#[cfg(feature = "julia")]
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn projected_page_family_search_matches_reference_family_clusters() {
    let analysis = sample_projection_analysis("projection-sample");
    let result = build_repo_projected_page_family_search(
        &RepoProjectedPageFamilySearchQuery {
            repo_id: "projection-sample".to_string(),
            query: "solve".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 5,
            per_kind_limit: 2,
        },
        &analysis,
    );

    assert_repo_json_snapshot("repo_projected_page_family_search_result", json!(result));
}

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_projected_page_family_search_matches_reference_family_clusters() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-family-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-family-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = repo_projected_page_family_search_from_config(
        &RepoProjectedPageFamilySearchQuery {
            repo_id: "modelica-family-search".to_string(),
            query: "Projectionica.Controllers".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 3,
            per_kind_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot(
        "repo_projected_page_family_search_modelica_result",
        json!(result),
    );
    Ok(())
}
