//! Integration tests for Repo Intelligence example search flow.

use crate::support::repo_intelligence::{
    analyze_repository_from_config_cached, assert_repo_json_snapshot,
    create_cached_sample_julia_repo, write_repo_config,
};
use serde_json::json;
use serial_test::serial;
use xiuxian_wendao::analyzers::{ExampleSearchQuery, build_example_search};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
#[serial(repo_intelligence_example_search)]
fn example_search_matches_related_symbol_name() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo("example-search", "ExamplePkg", true, &[])?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(config_root, &repo_dir, "example-sample")?;

    let analysis =
        analyze_repository_from_config_cached("example-sample", Some(&config_path), config_root)?;
    let result = build_example_search(
        &ExampleSearchQuery {
            repo_id: "example-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_repo_json_snapshot("repo_example_search_result", json!(result));
    Ok(())
}

#[test]
#[serial(repo_intelligence_example_search)]
fn example_search_exposes_ranked_hits_for_frontend_sorting() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo("example-search", "ExamplePkg", true, &[])?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(config_root, &repo_dir, "example-sample")?;

    let analysis =
        analyze_repository_from_config_cached("example-sample", Some(&config_path), config_root)?;
    let result = build_example_search(
        &ExampleSearchQuery {
            repo_id: "example-sample".to_string(),
            query: "solve".to_string(),
            limit: 10,
        },
        &analysis,
    );

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
#[serial(repo_intelligence_example_search)]
fn example_search_uses_shared_tantivy_fuzzy_index_for_title_typos() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo("example-search", "ExamplePkg", true, &[])?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(config_root, &repo_dir, "example-sample")?;

    let analysis =
        analyze_repository_from_config_cached("example-sample", Some(&config_path), config_root)?;
    let result = build_example_search(
        &ExampleSearchQuery {
            repo_id: "example-sample".to_string(),
            query: "basci".to_string(),
            limit: 10,
        },
        &analysis,
    );

    assert_eq!(result.examples.len(), 1);
    assert_eq!(result.examples[0].title, "basic");
    let Some(score) = result.example_hits[0].score else {
        panic!("shared fuzzy example search should emit a score");
    };
    assert!(score > 0.0);
    Ok(())
}

#[test]
#[serial(repo_intelligence_example_search)]
fn cli_repo_example_search_returns_serialized_result() -> TestResult {
    let repo_dir = create_cached_sample_julia_repo("example-search", "ExamplePkg", true, &[])?;
    let config_root = repo_dir.parent().unwrap_or(repo_dir.as_path());
    let config_path = write_repo_config(config_root, &repo_dir, "example-sample")?;

    let result = build_example_search(
        &ExampleSearchQuery {
            repo_id: "example-sample".to_string(),
            query: "test".to_string(),
            limit: 10,
        },
        &analyze_repository_from_config_cached("example-sample", Some(&config_path), config_root)?,
    );
    assert_repo_json_snapshot(
        "repo_example_search_cli_json",
        serde_json::to_value(result)?,
    );
    Ok(())
}
