//! Integration tests for deterministic docs-facing projected page family context.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsFamilyContextQuery, ProjectionPageKind, RepoProjectedPagesQuery,
    docs_family_context_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_family_context_groups_related_pages_by_family() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-family-context.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-family-context]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-docs-family-context".to_string(),
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

    let result = docs_family_context_from_config(
        &DocsFamilyContextQuery {
            repo_id: "modelica-docs-family-context".to_string(),
            page_id: page.page_id.clone(),
            per_kind_limit: 2,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_family_context_modelica_result", json!(result));
    Ok(())
}
