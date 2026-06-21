//! Integration tests for deterministic docs-facing projected-page search.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsSearchQuery, ProjectionPageKind, docs_search_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_search_matches_reference_pages() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_search_from_config(
        &DocsSearchQuery {
            repo_id: "modelica-docs-search".to_string(),
            query: "Projectionica.Controllers".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 3,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_search_modelica_result", json!(result));
    Ok(())
}
