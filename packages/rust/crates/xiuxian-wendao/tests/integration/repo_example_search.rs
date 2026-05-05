//! Integration tests for Repo Intelligence example search flow.

use crate::support::repo_intelligence::{
    assert_repo_json_snapshot, create_sample_julia_repo, write_repo_config,
};
use crate::support::wendao_command;
use serde_json::json;
use xiuxian_wendao::analyzers::{ExampleSearchQuery, example_search_from_config};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn example_search_matches_related_symbol_name() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ExamplePkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "example-sample")?;

    let result = example_search_from_config(
        &ExampleSearchQuery {
            repo_id: "example-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_repo_json_snapshot("repo_example_search_result", json!(result));
    Ok(())
}

#[test]
fn example_search_exposes_ranked_hits_for_frontend_sorting() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ExampleRankPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "example-rank-sample")?;

    let result = example_search_from_config(
        &ExampleSearchQuery {
            repo_id: "example-rank-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.examples.len(), result.example_hits.len());
    assert!(
        result
            .example_hits
            .iter()
            .enumerate()
            .all(|(index, hit)| hit.rank == Some(index + 1)),
        "example hit ranks should be contiguous and 1-based"
    );
    assert!(
        result.example_hits.iter().all(|hit| hit.score.is_some()),
        "example hit scores should be emitted by backend"
    );
    for hit in &result.example_hits {
        if let Some(items) = hit.implicit_backlink_items.as_ref() {
            assert_eq!(
                hit.implicit_backlinks.as_ref().map(Vec::len),
                Some(items.len()),
                "legacy backlink ids should stay aligned with structured backlink items"
            );
            assert!(
                items
                    .iter()
                    .all(|item| item.kind.as_deref() == Some("documents"))
            );
        }
    }
    Ok(())
}

#[test]
fn example_search_uses_shared_tantivy_fuzzy_index_for_title_typos() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "ExampleFuzzyPkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "example-fuzzy-sample")?;

    let result = example_search_from_config(
        &ExampleSearchQuery {
            repo_id: "example-fuzzy-sample".to_string(),
            query: "basci".to_string(),
            limit: 10,
        },
        Some(&config_path),
        temp.path(),
    )?;

    assert_eq!(result.examples.len(), 1);
    assert_eq!(result.examples[0].title, "basic");
    let Some(score) = result.example_hits[0].score else {
        panic!("shared fuzzy example search should emit a score");
    };
    assert!(score > 0.0);
    Ok(())
}

#[test]
fn cli_repo_example_search_returns_serialized_result() -> TestResult {
    let temp = tempfile::tempdir()?;
    let repo_dir = create_sample_julia_repo(temp.path(), "CliExamplePkg", true)?;
    let config_path = write_repo_config(temp.path(), &repo_dir, "cli-example")?;

    let output = wendao_command()
        .arg("--conf")
        .arg(&config_path)
        .arg("--output")
        .arg("json")
        .arg("repo")
        .arg("example-search")
        .arg("--repo")
        .arg("cli-example")
        .arg("--query")
        .arg("test")
        .output()?;

    assert!(output.status.success(), "{output:?}");

    let payload: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_repo_json_snapshot("repo_example_search_cli_json", payload);
    Ok(())
}
