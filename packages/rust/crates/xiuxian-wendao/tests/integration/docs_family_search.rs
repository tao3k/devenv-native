//! Integration tests for deterministic docs-facing projected page family search.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsFamilySearchQuery, ProjectionPageKind, docs_family_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_family_search_matches_reference_family_clusters() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-family-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-family-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_family_search_from_config(
        &DocsFamilySearchQuery {
            repo_id: "modelica-docs-family-search".to_string(),
            query: "Projectionica.Controllers".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 3,
            per_kind_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_family_search_modelica_result", json!(result));
    Ok(())
}
