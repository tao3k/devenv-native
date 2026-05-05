use crate::studio::arrow_types::LanceArray;

use crate::studio::router::handlers::repo::analysis::overview_flight::{
    build_repo_overview_flight_batch, build_repo_overview_flight_metadata,
};
use xiuxian_wendao::analyzers::RepoOverviewResult;

#[test]
fn repo_overview_flight_batch_preserves_summary_fields() {
    let batch = build_repo_overview_flight_batch(&RepoOverviewResult {
        repo_id: "gateway-sync".to_string(),
        display_name: "Gateway Sync".to_string(),
        revision: Some("rev:123".to_string()),
        module_count: 3,
        symbol_count: 8,
        example_count: 2,
        doc_count: 5,
        hierarchical_uri: Some("repo://gateway-sync".to_string()),
        hierarchy: Some(vec!["repo".to_string(), "gateway-sync".to_string()]),
    })
    .unwrap_or_else(|error| panic!("repo overview batch should build: {error}"));

    assert_eq!(batch.num_rows(), 1);
    let Some(display_name_column) = batch.column_by_name("displayName") else {
        panic!("displayName column");
    };
    let Some(display_name) = display_name_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceStringArray>()
    else {
        panic!("displayName should be utf8");
    };
    assert_eq!(display_name.value(0), "Gateway Sync");

    let Some(doc_count_column) = batch.column_by_name("docCount") else {
        panic!("docCount column");
    };
    let Some(doc_count) = doc_count_column
        .as_any()
        .downcast_ref::<crate::studio::arrow_types::LanceInt32Array>()
    else {
        panic!("docCount should be int32");
    };
    assert_eq!(doc_count.value(0), 5);
}

#[test]
fn repo_overview_flight_metadata_preserves_summary_fields() {
    let metadata = build_repo_overview_flight_metadata(&RepoOverviewResult {
        repo_id: "gateway-sync".to_string(),
        display_name: "Gateway Sync".to_string(),
        revision: Some("rev:123".to_string()),
        module_count: 3,
        symbol_count: 8,
        example_count: 2,
        doc_count: 5,
        hierarchical_uri: Some("repo://gateway-sync".to_string()),
        hierarchy: Some(vec!["repo".to_string(), "gateway-sync".to_string()]),
    })
    .unwrap_or_else(|error| panic!("repo overview metadata should encode: {error}"));

    let payload: serde_json::Value = serde_json::from_slice(&metadata)
        .unwrap_or_else(|error| panic!("metadata should decode: {error}"));
    assert_eq!(payload["repoId"], "gateway-sync");
    assert_eq!(payload["displayName"], "Gateway Sync");
    assert_eq!(payload["revision"], "rev:123");
    assert_eq!(payload["moduleCount"], 3);
    assert_eq!(payload["symbolCount"], 8);
    assert_eq!(payload["exampleCount"], 2);
    assert_eq!(payload["docCount"], 5);
    assert_eq!(payload["hierarchicalUri"], "repo://gateway-sync");
    assert_eq!(payload["hierarchy"][0], "repo");
}
