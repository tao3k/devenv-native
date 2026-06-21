//! Integration tests for deterministic docs-facing mixed projected retrieval.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsRetrievalQuery, ProjectionPageKind, docs_retrieval_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_retrieval_merges_page_and_node_hits() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp.path().join("modelica-docs-retrieval.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-retrieval]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_retrieval_from_config(
        &DocsRetrievalQuery {
            repo_id: "modelica-docs-retrieval".to_string(),
            query: "Projectionica.Controllers".to_string(),
            kind: Some(ProjectionPageKind::Reference),
            limit: 4,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("docs_retrieval_modelica_result", json!(result));
    Ok(())
}
