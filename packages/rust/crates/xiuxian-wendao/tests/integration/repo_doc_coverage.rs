//! Integration tests for Repo Intelligence documentation coverage flow.

use crate::support::repo_intelligence::{
    analyze_repository_from_config_cached, assert_repo_json_snapshot,
    create_cached_sample_julia_repo, write_repo_config,
};
use serde_json::json;
use serial_test::serial;
use xiuxian_wendao::analyzers::{DocCoverageQuery, build_doc_coverage};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
#[serial(repo_intelligence_doc_coverage)]
fn doc_coverage_counts_symbol_specific_docs_for_module_scope() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo(
        "doc-coverage-symbol",
        "CoveragePkg",
        true,
        &[
            ("docs/Problem.md", "# Problem\n"),
            ("docs/solve.md", "# solve\n"),
        ],
    )?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(&repo_dir, &repo_dir, "coverage-sample")?;
    let analysis =
        analyze_repository_from_config_cached("coverage-sample", Some(&config_path), config_root)?;
    let module = analysis
        .modules
        .first()
        .ok_or("expected one module in analysis output")?;

    let result = build_doc_coverage(
        &DocCoverageQuery {
            repo_id: "coverage-sample".to_string(),
            module_id: Some(module.qualified_name.clone()),
        },
        &analysis,
    );

    assert_repo_json_snapshot("repo_doc_coverage_result", json!(result));
    Ok(())
}

#[test]
#[serial(repo_intelligence_doc_coverage)]
fn cli_repo_doc_coverage_returns_serialized_result() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo("doc-coverage-cli", "CoveragePkg", true, &[])?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(config_root, &repo_dir, "coverage-sample")?;

    let output = build_doc_coverage(
        &DocCoverageQuery {
            repo_id: "coverage-sample".to_string(),
            module_id: None,
        },
        &analyze_repository_from_config_cached("coverage-sample", Some(&config_path), config_root)?,
    );
    assert_repo_json_snapshot("repo_doc_coverage_cli_json", serde_json::to_value(output)?);
    Ok(())
}
