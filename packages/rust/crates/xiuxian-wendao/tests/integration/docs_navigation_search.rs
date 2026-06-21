//! Integration tests for deterministic docs-facing projected page navigation search.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsNavigationSearchQuery, ProjectionPageKind, docs_navigation_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_navigation_search_expands_reference_hits_into_navigation_bundles()
-> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-navigation-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-navigation-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_navigation_search_from_config(
        &DocsNavigationSearchQuery {
            repo_id: "modelica-docs-navigation-search".to_string(),
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

    assert_repo_json_snapshot("docs_navigation_search_modelica_result", json!(result));
    Ok(())
}
