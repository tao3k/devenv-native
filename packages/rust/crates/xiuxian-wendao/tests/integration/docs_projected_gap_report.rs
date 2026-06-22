//! Integration tests for deterministic docs-facing projected deep-wiki gap reporting.
#![cfg(feature = "julia")]

use std::fs;

use crate::support::repo_intelligence::{assert_repo_json_snapshot, create_sample_modelica_repo};
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocsProjectedGapReportQuery, docs_projected_gap_report_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[cfg(feature = "julia")]
#[test]
fn modelica_plugin_docs_projected_gap_report_executes_over_external_plugin_path() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Projectionica")?;
    let config_path = temp
        .path()
        .join("modelica-docs-projected-gap-report.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[sources.projects.modelica-docs-projected-gap-report]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = docs_projected_gap_report_from_config(
        &DocsProjectedGapReportQuery {
            repo_id: "modelica-docs-projected-gap-report".to_string(),
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(
        result.summary.gap_count,
        result.gaps.len(),
        "gap summary should match materialized gap count"
    );
    assert_repo_json_snapshot("docs_projected_gap_report_modelica_result", json!(result));
    Ok(())
}
