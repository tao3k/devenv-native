use super::{
    CODE_INTELLIGENCE_STRUCTURED_CANDIDATE_COUNT, FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
    MARKDOWN_HEADING_CANDIDATE_SOURCE, PRIMARY_MARKDOWN_STRUCTURED_CANDIDATE_COUNT,
    REGISTRY_AUTHORITY_STRUCTURED_CANDIDATE_COUNT, RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
    TOTAL_STRUCTURED_CANDIDATE_COUNT, search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_total_structured_candidate_index_contract,
};

#[test]
fn total_structured_candidate_index_contract_uses_2818_promotion_denominator() {
    let contract = search_strategy_flow_total_structured_candidate_index_contract();

    assert_eq!(contract.surfaces.len(), 3);
    assert_eq!(
        contract.total_candidate_count(),
        TOTAL_STRUCTURED_CANDIDATE_COUNT
    );
    assert_eq!(
        contract
            .surfaces
            .iter()
            .find(|surface| surface.surface_id == "primary-markdown")
            .map(|surface| surface.candidate_count),
        Some(PRIMARY_MARKDOWN_STRUCTURED_CANDIDATE_COUNT)
    );
    assert_eq!(
        contract
            .surfaces
            .iter()
            .find(|surface| surface.surface_id == "code-intelligence-downlink")
            .map(|surface| surface.candidate_count),
        Some(CODE_INTELLIGENCE_STRUCTURED_CANDIDATE_COUNT)
    );
    assert_eq!(
        contract
            .surfaces
            .iter()
            .find(|surface| surface.surface_id == "registry-authority")
            .map(|surface| surface.candidate_count),
        Some(REGISTRY_AUTHORITY_STRUCTURED_CANDIDATE_COUNT)
    );

    assert!(contract.all_surfaces_share_rust_backend());
    assert!(contract.all_surfaces_use_narrowed_julia_batches());
    assert!(contract.all_surfaces_are_promotion_denominator());
    assert_eq!(contract.pending_surface_count(), 2);
    assert!(
        contract
            .surfaces
            .iter()
            .all(|surface| surface.rust_backend == RUST_DUCKDB_STRUCTURED_INDEX_BACKEND)
    );
}

#[test]
fn candidate_discovery_contract_maps_flight_source_to_total_structured_denominator() {
    let summary = search_strategy_flow_candidate_discovery_contract_json(
        Some(FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE),
        12,
        Some(&serde_json::json!({
            "transport": "arrow-flight",
            "route": "/search/repos/main",
            "attemptCount": 2,
        })),
    );

    assert_eq!(
        summary.get("candidateInputSource"),
        Some(&serde_json::json!(FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE))
    );
    assert_eq!(
        summary.get("candidateInputCount"),
        Some(&serde_json::json!(12))
    );
    assert_eq!(
        summary.get("promotionDenominator"),
        Some(&serde_json::json!(TOTAL_STRUCTURED_CANDIDATE_COUNT))
    );
    assert_eq!(
        summary.get("structuredSurfaceId"),
        Some(&serde_json::json!("code-intelligence-downlink"))
    );
    assert_eq!(
        summary.get("structuredSurfaceCandidateCount"),
        Some(&serde_json::json!(
            CODE_INTELLIGENCE_STRUCTURED_CANDIDATE_COUNT
        ))
    );
    assert_eq!(
        summary.get("selectionPolicy"),
        Some(&serde_json::json!("narrowed-candidate-batch"))
    );
    assert_eq!(
        summary.get("inputIsNarrowedBatch"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        summary.get("juliaReceivesFullInventory"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        summary
            .get("discoveryReceipt")
            .and_then(|receipt| receipt.get("transport")),
        Some(&serde_json::json!("arrow-flight"))
    );
}

#[test]
fn candidate_discovery_contract_maps_markdown_source_as_subset_not_denominator() {
    let summary = search_strategy_flow_candidate_discovery_contract_json(
        Some(MARKDOWN_HEADING_CANDIDATE_SOURCE),
        12,
        None,
    );

    assert_eq!(
        summary.get("structuredSurfaceId"),
        Some(&serde_json::json!("primary-markdown"))
    );
    assert_eq!(
        summary.get("structuredSurfaceCandidateCount"),
        Some(&serde_json::json!(
            PRIMARY_MARKDOWN_STRUCTURED_CANDIDATE_COUNT
        ))
    );
    assert_eq!(
        summary.get("totalStructuredCandidateCount"),
        Some(&serde_json::json!(TOTAL_STRUCTURED_CANDIDATE_COUNT))
    );
    assert_eq!(
        summary.get("inputSourceInStructuredContract"),
        Some(&serde_json::json!(true))
    );
}
