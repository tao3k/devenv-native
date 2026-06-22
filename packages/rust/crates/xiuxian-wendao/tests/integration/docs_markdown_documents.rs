//! Integration tests for deterministic docs-facing projected markdown documents.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{DocsMarkdownDocumentsQuery, docs_markdown_documents_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_markdown_documents_resolve_projected_markdown_documents() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-markdown-documents.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-markdown-documents]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_markdown_documents_from_config(
        &DocsMarkdownDocumentsQuery {
            repo_id: "modelica-docs-markdown-documents".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_markdown_documents_modelica_result", json!(result));
    Ok(())
}
