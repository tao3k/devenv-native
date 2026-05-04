//! Integration tests for Repo Intelligence documentation coverage flow.

use std::fs;

use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use crate::support::wendao_command;
use serde_json::json;
use xiuxian_wendao::analyzers::{
    DocCoverageQuery, analyze_repository_from_config, doc_coverage_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn doc_coverage_counts_symbol_specific_docs_for_module_scope() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CoveragePkg", true)?;
    fs::write(repo_dir.join("docs").join("Problem.md"), "# Problem\n")?;
    fs::write(repo_dir.join("docs").join("solve.md"), "# solve\n")?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "coverage-sample")?;
    let analysis =
        analyze_repository_from_config("coverage-sample", Some(&config_path), temp.path())?;
    let module = analysis
        .modules
        .first()
        .ok_or("expected one module in analysis output")?;

    let result = doc_coverage_from_config(
        &DocCoverageQuery {
            repo_id: "coverage-sample".to_string(),
            module_id: Some(module.qualified_name.clone()),
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_doc_coverage_result", json!(result));
    Ok(())
}

#[test]
fn cli_repo_doc_coverage_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliCoveragePkg", true)?;
    fs::write(repo_dir.join("docs").join("Problem.md"), "# Problem\n")?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-coverage")?;

    let output = wendao_command()
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("doc-coverage")
        .arg("--repo")
        .arg("cli-coverage")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_repo_json_snapshot("repo_doc_coverage_cli_json", payload);
    Ok(())
}
