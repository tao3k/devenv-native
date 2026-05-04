//! Integration tests for Repo Intelligence symbol search flow.

use std::fs;

use crate::support::repo_intelligence::create_sample_modelica_repo;
use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, sample_projection_analysis,
    write_repo_config,
};
use crate::support::wendao_command;
use serde_json::json;
use xiuxian_wendao::analyzers::{
    SymbolRecord, SymbolSearchQuery, build_symbol_search, symbol_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn symbol_search_matches_symbol_name() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "SymbolPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "symbol-sample")?;

    let result = symbol_search_from_config(
        &SymbolSearchQuery {
            repo_id: "symbol-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_symbol_search_result", json!(result));
    Ok(())
}

#[test]
fn modelica_plugin_symbol_search_matches_external_symbols() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_modelica_repo(temp.path(), "Symbolica")?;
    let config_path = temp.path().join("modelica-symbol-search.wendao.toml");
    fs::write(
        &config_path,
        format!(
            r#"[link_graph.projects.modelica-symbol-search]
root = "{}"
plugins = ["modelica"]
"#,
            repo_dir.display()
        ),
    )?;

    let result = symbol_search_from_config(
        &SymbolSearchQuery {
            repo_id: "modelica-symbol-search".to_string(),
            query: "PI".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_symbol_search_modelica_result", json!(result));
    Ok(())
}

#[test]
fn symbol_search_exposes_ranked_hits_for_frontend_sorting() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "SymbolRankPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "symbol-rank-sample")?;

    let result = symbol_search_from_config(
        &SymbolSearchQuery {
            repo_id: "symbol-rank-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.symbols.len(), result.symbol_hits.len());
    assert!(
        result
            .symbol_hits
            .iter()
            .enumerate()
            .all(|(index, hit)| hit.rank == Some(index + 1)),
        "symbol hit ranks should be contiguous and 1-based"
    );
    assert!(
        result.symbol_hits.iter().all(|hit| hit.score.is_some()),
        "symbol hit scores should be emitted by backend"
    );
    let backlink_item = result
        .symbol_hits
        .iter()
        .flat_map(|hit| {
            hit.implicit_backlink_items
                .iter()
                .flat_map(|items| items.iter())
        })
        .next();
    assert!(
        backlink_item.is_some(),
        "symbol hit backlink items should expose structured metadata when relations exist"
    );
    let Some(backlink_item) = backlink_item else {
        panic!("backlink item should exist");
    };
    assert!(!backlink_item.id.trim().is_empty());
    assert_eq!(backlink_item.kind.as_deref(), Some("documents"));
    Ok(())
}

#[test]
fn symbol_record_deserializes_legacy_and_audit_payloads() -> TestResult {
    let legacy = json!({
        "repo_id": "demo",
        "symbol_id": "repo:demo:symbol:Demo.solve",
        "module_id": "repo:demo:module:Demo",
        "name": "solve",
        "qualified_name": "Demo.solve",
        "kind": "function",
        "path": "src/Demo.jl",
        "signature": "solve() = nothing"
    });
    let parsed_legacy: SymbolRecord = serde_json::from_value(legacy)?;
    assert_eq!(parsed_legacy.audit_status, None);

    let with_audit = json!({
        "repo_id": "demo",
        "symbol_id": "repo:demo:symbol:Demo.solve",
        "module_id": "repo:demo:module:Demo",
        "name": "solve",
        "qualified_name": "Demo.solve",
        "kind": "function",
        "path": "src/Demo.jl",
        "signature": "solve() = nothing",
        "audit_status": "verified"
    });
    let parsed_with_audit: SymbolRecord = serde_json::from_value(with_audit)?;
    assert_eq!(parsed_with_audit.audit_status.as_deref(), Some("verified"));
    Ok(())
}

#[test]
fn cli_repo_symbol_search_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliSymbolPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-symbol")?;

    let output = wendao_command()
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("symbol-search")
        .arg("--repo")
        .arg("cli-symbol")
        .arg("--query")
        .arg("Problem")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_repo_json_snapshot("repo_symbol_search_cli_json", payload);
    Ok(())
}

#[test]
fn symbol_search_uses_shared_tantivy_fuzzy_index_for_typos() {
    let analysis = sample_projection_analysis("symbol-fuzzy");
    let result = build_symbol_search(
        &SymbolSearchQuery {
            repo_id: "symbol-fuzzy".to_string(),
            query: "slove".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.symbols.len(), 1);
    assert_eq!(result.symbols[0].name, "solve");
    let Some(score) = result.symbol_hits[0].score else {
        panic!("shared fuzzy search should emit a score");
    };
    assert!(score > 0.0);
}
