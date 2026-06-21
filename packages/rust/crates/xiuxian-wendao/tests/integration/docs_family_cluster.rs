//! Integration tests for deterministic docs-facing projected page family cluster lookup.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsFamilyClusterQuery, ProjectionPageKind, RepoProjectedPagesQuery,
    docs_family_cluster_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_family_cluster_resolves_how_to_cluster_for_reference_page() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-family-cluster.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-family-cluster]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-docs-family-cluster".to_string(),
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

    let result = docs_family_cluster_from_config(
        &DocsFamilyClusterQuery {
            repo_id: "modelica-docs-family-cluster".to_string(),
            page_id: page.page_id.clone(),
            kind: ProjectionPageKind::HowTo,
            limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_family_cluster_modelica_result", json!(result));
    Ok(())
}
