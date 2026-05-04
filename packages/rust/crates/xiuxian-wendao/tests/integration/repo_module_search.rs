//! Integration tests for Repo Intelligence module search flow.

use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, sample_projection_analysis,
    write_repo_config,
};
use crate::support::wendao_command;
use serde_json::json;
use xiuxian_wendao::analyzers::{
    ModuleSearchQuery, build_module_search, module_search_from_config,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn module_search_matches_qualified_name() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ModulePkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "module-sample")?;

    let result = module_search_from_config(
        &ModuleSearchQuery {
            repo_id: "module-sample".to_string(),
            query: "ModulePkg".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_module_search_result", json!(result));
    Ok(())
}

#[test]
fn module_search_exposes_ranked_hits_for_frontend_sorting() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ModuleRankPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "module-rank-sample")?;

    let result = module_search_from_config(
        &ModuleSearchQuery {
            repo_id: "module-rank-sample".to_string(),
            query: "module".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.modules.len(), result.module_hits.len());
    assert!(
        result
            .module_hits
            .iter()
            .enumerate()
            .all(|(index, hit)| hit.rank == Some(index + 1)),
        "module hit ranks should be contiguous and 1-based"
    );
    assert!(
        result.module_hits.iter().all(|hit| hit.score.is_some()),
        "module hit scores should be emitted by backend"
    );
    let backlink_item = result
        .module_hits
        .iter()
        .flat_map(|hit| {
            hit.implicit_backlink_items
                .iter()
                .flat_map(|items| items.iter())
        })
        .next();
    assert!(
        backlink_item.is_some(),
        "module hit backlink items should expose structured metadata when relations exist"
    );
    let Some(backlink_item) = backlink_item else {
        panic!("backlink item should exist");
    };
    assert!(!backlink_item.id.trim().is_empty());
    assert_eq!(backlink_item.kind.as_deref(), Some("documents"));
    Ok(())
}

#[test]
fn cli_repo_module_search_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliModulePkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-module")?;

    let output = wendao_command()
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("module-search")
        .arg("--repo")
        .arg("cli-module")
        .arg("--query")
        .arg("src")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_repo_json_snapshot("repo_module_search_cli_json", payload);
    Ok(())
}

#[test]
fn module_search_uses_shared_tantivy_fuzzy_index_for_typos() {
    let analysis = sample_projection_analysis("module-fuzzy");
    let result = build_module_search(
        &ModuleSearchQuery {
            repo_id: "module-fuzzy".to_string(),
            query: "ProjectonPkg".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.modules.len(), 1);
    assert_eq!(result.modules[0].qualified_name, "ProjectionPkg");
    let Some(score) = result.module_hits[0].score else {
        panic!("shared fuzzy search should emit a score");
    };
    assert!(score > 0.0);
}
