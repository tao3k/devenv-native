use super::{
    REPO_SEARCH_ROUTE, SearchStrategyFlowFlightMaterializationConfig,
    search_strategy_flow_candidate_input_batch_from_repo_search,
    spawn_fake_search_strategy_flow_candidate_discovery_service,
};
use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

#[tokio::test]
async fn search_strategy_flow_flight_candidate_discovery_decodes_non_markdown_source_config_rows() {
    let (base_url, server) = spawn_fake_search_strategy_flow_candidate_discovery_service().await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight candidate config: {error}"));

    let batch = search_strategy_flow_candidate_input_batch_from_repo_search(
        "search strategy flow link graph python julia toml",
        &config,
    )
    .await
    .unwrap_or_else(|error| panic!("discover fake Flight candidates: {error}"));
    server.abort();

    assert_eq!(batch.source, "wendao-gateway-retrieval");
    assert_eq!(batch.row_count, 3);
    assert_eq!(batch.candidate_input_arrow_snapshot().lines().count(), 3);
    let discovery_receipt: serde_json::Value =
        serde_json::from_str(batch.discovery_receipt_json.as_str())
            .unwrap_or_else(|error| panic!("parse fake Flight discovery receipt: {error}"));
    assert_eq!(
        discovery_receipt.get("transport"),
        Some(&serde_json::json!(WENDAO_ARROW_FLIGHT_DATA_PLANE))
    );
    assert_eq!(
        discovery_receipt.get("retrievalOwner"),
        Some(&serde_json::json!("wendao-gateway"))
    );
    assert_eq!(
        discovery_receipt.get("route"),
        Some(&serde_json::json!(REPO_SEARCH_ROUTE))
    );
    assert_eq!(
        discovery_receipt.get("candidateInputCount"),
        Some(&serde_json::json!(3))
    );
    assert_eq!(
        discovery_receipt.get("mergedCandidateCount"),
        Some(&serde_json::json!(3))
    );
    assert!(
        discovery_receipt
            .get("attempts")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|attempts| {
                attempts.iter().any(|attempt| {
                    attempt.get("route") == Some(&serde_json::json!(REPO_SEARCH_ROUTE))
                        && attempt.get("rowCount") == Some(&serde_json::json!(4))
                })
            }),
        "fake Flight discovery receipt must keep per-attempt row counts: {discovery_receipt}"
    );
    for expected_fragment in [
        "packages/rust/crates/xiuxian-wendao/src/link_graph/index/ppr/mod.rs",
        "wendao.toml",
        "packages/python/xiuxian-wendao-analyzer/src/xiuxian_wendao_analyzer/worker.py",
        "ppr-runtime-search-strategy",
        "wendao-repository-configuration",
        "python-analyzer-worker",
        "effective-parser:rust-lang-parser",
        "effective-parser:xiuxian-ast:toml",
        "effective-parser:xiuxian-ast:python",
        "parser-priority:local-override",
        "parser-priority:general-baseline",
        "repo-search",
        WENDAO_ARROW_FLIGHT_DATA_PLANE,
    ] {
        assert!(
            batch
                .candidate_input_arrow_snapshot()
                .contains(expected_fragment),
            "candidate batch should contain `{expected_fragment}`:\n{}",
            batch.candidate_input_arrow_snapshot()
        );
    }
    assert!(
        !batch
            .candidate_input_arrow_snapshot()
            .contains(".data/WendaoGraph.jl"),
        "Flight candidate discovery must not promote transient nested-repo paths:\n{}",
        batch.candidate_input_arrow_snapshot()
    );
}
