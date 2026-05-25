use std::fmt::Write as _;

use xiuxian_wendao_core::repo_intelligence::{RegisteredRepository, RepositoryPluginConfig};

use super::{
    fetch_julia_parser_file_summary_for_repository, fetch_julia_parser_root_summary_for_repository,
    shared_julia_parser_summary_runtime_identity_for_tests,
    validate_julia_parser_summary_preflight_for_repository,
};
use crate::julia_plugin_test_support::common::ensure_linked_julia_parser_summary_service;

const JULIA_LARGE_FILE_SUMMARY_TARGET_BYTES: usize = 32 * 1024;
const JULIA_LARGE_SPARSE_FILE_SUMMARY_TARGET_BYTES: usize = 32 * 1024;
const JULIA_CONCURRENT_FILE_SUMMARY_TARGET_BYTES: usize = 16 * 1024;
const JULIA_CONCURRENT_FILE_SUMMARY_REQUESTS: usize = 2;
const JULIA_SYNTHETIC_SYMBOL_COUNT: usize = 12;

fn parser_summary_repository() -> RegisteredRepository {
    RegisteredRepository {
        id: "repo-julia".to_string(),
        plugins: vec![RepositoryPluginConfig::Id("julia-code-parser".to_string())],
        ..RegisteredRepository::default()
    }
}

fn synthetic_large_julia_module(target_bytes: usize) -> String {
    let mut source = String::from("module StressDemo\nexport solve_0\n\n");
    for index in 0..JULIA_SYNTHETIC_SYMBOL_COUNT {
        write!(
            source,
            "function solve_{index}(x)\n    x + {index}\nend\n\nconst VALUE_{index} = {index}\n\n"
        )
        .unwrap_or_else(|error| panic!("append synthetic Julia source: {error}"));
    }
    while source.len() < target_bytes {
        source.push_str(
            "# filler line to expand request size while preserving a bounded symbol surface\n",
        );
    }
    source.push_str("end\n");
    source
}

fn synthetic_large_sparse_julia_module(target_bytes: usize) -> String {
    let mut source = String::from("module SparseStressDemo\nexport solve\n\n");
    while source.len() < target_bytes {
        source.push_str("# filler line to expand request size without expanding summary rows\n");
    }
    source.push_str("\nsolve(x) = x\nend\n");
    source
}

#[test]
fn parser_summary_preflight_accepts_plain_plugin_default_discovery() {
    let repository = parser_summary_repository();

    validate_julia_parser_summary_preflight_for_repository(&repository).unwrap_or_else(|error| {
        panic!("plain Julia plugin id should resolve parser summary: {error}")
    });
}

#[test]
fn blocking_fetch_uses_shared_julia_parser_summary_runtime() {
    let first = shared_julia_parser_summary_runtime_identity_for_tests()
        .unwrap_or_else(|error| panic!("first shared Julia parser-summary runtime: {error}"));
    let second = shared_julia_parser_summary_runtime_identity_for_tests()
        .unwrap_or_else(|error| panic!("second shared Julia parser-summary runtime: {error}"));

    assert_eq!(first, second);
}

#[tokio::test]
async fn fetch_parser_summaries_against_linked_real_wendaosearch_service()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    let repository = parser_summary_repository();
    let source = r#"module Demo
export solve
using ..Core: solve as solver
include("solvers.jl")

"""
Solve docs.
"""
function solve(problem::Problem)
    problem.x
end

const LIMIT = 1
end
"#;

    let summary =
        fetch_julia_parser_file_summary_for_repository(&repository, "src/Demo.jl", source)
            .await
            .unwrap_or_else(|error| panic!("file summary fetch should succeed: {error}"));

    assert_eq!(summary.module_name.as_deref(), Some("Demo"));
    assert_eq!(summary.exports, vec!["solve".to_string()]);
    assert_eq!(summary.includes, vec!["solvers.jl".to_string()]);
    assert_eq!(summary.imports.len(), 1);
    assert_eq!(summary.imports[0].module, "..Core.solve".to_string());
    assert!(summary.imports[0].dependency_is_relative);
    assert_eq!(
        summary.imports[0].dependency_alias.as_deref(),
        Some("solver")
    );
    assert!(
        summary
            .symbols
            .iter()
            .any(|symbol| symbol.name == "solve" && symbol.signature.is_some()),
        "missing `solve` symbol: {:?}",
        summary.symbols,
    );
    assert!(
        summary.symbols.iter().any(|symbol| symbol.name == "LIMIT"),
        "missing `LIMIT` binding: {:?}",
        summary.symbols,
    );
    assert!(
        summary
            .docstrings
            .iter()
            .any(|doc| doc.target_name == "solve" && doc.content == "Solve docs."),
        "missing `solve` docstring: {:?}",
        summary.docstrings,
    );
    let root_summary = fetch_julia_parser_root_summary_for_repository(
        &repository,
        "src/standalone.jl",
        "solve(x) = x\n",
    )
    .await;
    let Err(error) = root_summary else {
        panic!("root summary without module must fail");
    };

    assert!(
        error
            .to_string()
            .contains("Julia root summary requires one root module declaration"),
        "unexpected error: {error}",
    );

    Ok(())
}

#[tokio::test]
async fn fetch_large_parser_file_summary_against_linked_real_service()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    let repository = parser_summary_repository();
    let source = synthetic_large_julia_module(JULIA_LARGE_FILE_SUMMARY_TARGET_BYTES);

    let summary =
        fetch_julia_parser_file_summary_for_repository(&repository, "src/StressDemo.jl", &source)
            .await
            .unwrap_or_else(|error| panic!("large file summary fetch should succeed: {error}"));

    assert_eq!(summary.module_name.as_deref(), Some("StressDemo"));
    assert!(
        summary
            .symbols
            .iter()
            .any(|symbol| symbol.name == "solve_0"),
        "missing first synthetic symbol: {:?}",
        summary.symbols,
    );

    Ok(())
}

#[tokio::test]
async fn fetch_large_sparse_parser_file_summary_against_linked_real_service()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    let repository = parser_summary_repository();
    let source = synthetic_large_sparse_julia_module(JULIA_LARGE_SPARSE_FILE_SUMMARY_TARGET_BYTES);

    let summary = fetch_julia_parser_file_summary_for_repository(
        &repository,
        "src/SparseStressDemo.jl",
        &source,
    )
    .await
    .unwrap_or_else(|error| panic!("large sparse file summary fetch should succeed: {error}"));

    assert_eq!(summary.module_name.as_deref(), Some("SparseStressDemo"));
    assert!(
        summary.symbols.iter().any(|symbol| symbol.name == "solve"),
        "missing sparse synthetic symbol: {:?}",
        summary.symbols,
    );

    Ok(())
}

#[tokio::test]
async fn fetch_parser_file_summaries_concurrently_against_linked_real_service()
-> Result<(), Box<dyn std::error::Error>> {
    ensure_linked_julia_parser_summary_service()?;
    let repository = parser_summary_repository();
    let source = synthetic_large_julia_module(JULIA_CONCURRENT_FILE_SUMMARY_TARGET_BYTES);

    let mut tasks = tokio::task::JoinSet::new();
    for index in 0..JULIA_CONCURRENT_FILE_SUMMARY_REQUESTS {
        let repository = repository.clone();
        let source = source.clone();
        tasks.spawn(async move {
            fetch_julia_parser_file_summary_for_repository(
                &repository,
                format!("src/StressDemo{index}.jl").as_str(),
                source.as_str(),
            )
            .await
        });
    }

    while let Some(result) = tasks.join_next().await {
        let summary = result.unwrap_or_else(|error| {
            panic!("concurrent Julia parser-summary task should not panic: {error}")
        })?;
        assert_eq!(summary.module_name.as_deref(), Some("StressDemo"));
        assert!(
            summary
                .symbols
                .iter()
                .any(|symbol| symbol.name == "solve_0"),
            "missing first synthetic symbol: {:?}",
            summary.symbols,
        );
    }

    Ok(())
}
