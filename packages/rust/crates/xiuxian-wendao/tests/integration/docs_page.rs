//! Integration tests for deterministic docs-facing projected-page lookup.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsPageQuery, RepoProjectedPagesQuery, docs_page_from_config, repo_projected_pages_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_page_lookup_resolves_one_stable_page() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-page.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-page]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let pages = repo_projected_pages_from_config(
        &RepoProjectedPagesQuery {
            repo_id: "modelica-docs-page".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;
    let Some(page_id) = pages
        .pages
        .iter()
        .find(|page| {
            page.page_id.contains(":symbol:") && page.title == "Projectionica.Controllers.PI"
        })
        .map(|page| page.page_id.clone())
    else {
        panic!("expected a projected page for Projectionica.Controllers.PI");
    };

    let result = docs_page_from_config(
        &DocsPageQuery {
            repo_id: "modelica-docs-page".to_string(),
            page_id,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_page_modelica_result", json!(result));
    Ok(())
}
