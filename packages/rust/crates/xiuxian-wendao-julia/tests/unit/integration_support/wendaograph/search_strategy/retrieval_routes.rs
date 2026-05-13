use super::{
    enrich_wendaograph_search_strategy_flow_retrieval_routes,
    search_strategy_flow_live_replay_search_root,
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "JSON bridge fixture is intentionally explicit"
)]
fn search_strategy_flow_rust_bridge_adds_planned_retrieval_routes() {
    let search_root = search_strategy_flow_live_replay_search_root();
    let trace = serde_json::json!({
        "intent": "find query understanding",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": search_root,
        "candidateInputSource": "rust-code-intelligence-inventory",
        "candidateInputCount": 2,
        "candidateInputDiscovery": {
            "receiptSource": "rust-code-intelligence-inventory",
            "transport": "arrow-flight",
            "route": "/search/repos/main",
            "attemptCount": 2,
            "mergedCandidateCount": 2
        },
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "action": "keep",
                "reason": "score",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            },
            {
                "candidateId": "docs/90_validation/90.01_validation.md#promotion-boundary",
                "action": "prune",
                "reason": "blocked",
                "finalScore": 0.2,
                "evidenceCoverage": 0.1,
                "graphScore": 0.1,
                "authorityScore": 0.1,
                "semanticScore": 0.0,
                "structuralScore": 0.1,
                "contextCost": 100,
                "blocked": true
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "rank": 1,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });

    let enriched = enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace.to_string())
        .unwrap_or_else(|error| panic!("enrich SearchStrategyFlow bridge trace: {error}"));
    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse enriched SearchStrategyFlow bridge trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    let projected_rows = enriched
        .get("rustProjectedEvidenceRows")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("rustProjectedEvidenceRows must be an array"));
    let structured_contract = enriched
        .get("structuredCandidateIndexContract")
        .unwrap_or_else(|| panic!("structuredCandidateIndexContract must be present"));
    let discovery_contract = enriched
        .get("candidateDiscoveryContract")
        .unwrap_or_else(|| panic!("candidateDiscoveryContract must be present"));

    assert_eq!(routes.len(), 1);
    assert_eq!(projected_rows.len(), 2);
    assert_eq!(
        structured_contract.get("totalCandidateCount"),
        Some(&serde_json::json!(2))
    );
    assert_eq!(
        structured_contract.get("inventorySource"),
        Some(&serde_json::json!("gateway-flight-trace"))
    );
    assert_eq!(
        structured_contract.get("juliaInputPolicy"),
        Some(&serde_json::json!("narrowed-candidate-batch"))
    );
    assert_eq!(
        structured_contract.get("allSurfacesShareRustBackend"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        structured_contract
            .get("surfaces")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        discovery_contract.get("candidateInputSource"),
        Some(&serde_json::json!("rust-code-intelligence-inventory"))
    );
    assert_eq!(
        discovery_contract.get("structuredSurfaceId"),
        Some(&serde_json::json!("code-intelligence-downlink"))
    );
    assert_eq!(
        discovery_contract.get("promotionDenominator"),
        Some(&serde_json::json!(2))
    );
    assert_eq!(
        discovery_contract.get("inputIsNarrowedBatch"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        discovery_contract.get("juliaReceivesFullInventory"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        discovery_contract
            .get("discoveryReceipt")
            .and_then(|receipt| receipt.get("transport")),
        Some(&serde_json::json!("arrow-flight"))
    );
    assert_eq!(
        discovery_contract
            .get("discoveryReceipt")
            .and_then(|receipt| receipt.get("route")),
        Some(&serde_json::json!("/search/repos/main"))
    );
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("planned"))
    );
    assert_eq!(
        route.get("receiptSource"),
        Some(&serde_json::json!("rust-bridge"))
    );
    assert_eq!(
        route.get("sourcePath"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        route.get("headingAnchor"),
        Some(&serde_json::json!("stage-1-query-understanding"))
    );
    assert!(route.get("materializedRows").is_none());
    assert!(route.get("routeReceipts").is_none());
    assert_eq!(
        route
            .get("flightSteps")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(4)
    );
    let graph_step = route
        .get("flightSteps")
        .and_then(serde_json::Value::as_array)
        .and_then(|steps| steps.last())
        .unwrap_or_else(|| panic!("graph flight step"));
    assert_eq!(
        graph_step
            .get("requiresResolvedGraphNodeId")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert!(
        serde_json::to_string(graph_step)
            .unwrap_or_else(|error| panic!("graph flight step should serialize: {error}"))
            .contains("<resolved-graph-node-id>")
    );

    let selected_evidence = projected_rows
        .iter()
        .find(|row| {
            row.get("candidateId")
                == Some(&serde_json::json!(
                    "docs/30_search_strategy/30.01_search_strategy_flow.md#stage-1-query-understanding"
                ))
        })
        .unwrap_or_else(|| panic!("selected projected evidence row"));
    assert_eq!(
        selected_evidence.get("projectionSource"),
        Some(&serde_json::json!("rust-bridge-search-strategy-flow-v1"))
    );
    assert_eq!(
        selected_evidence.get("sourcePath"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        selected_evidence.get("headingAnchor"),
        Some(&serde_json::json!("stage-1-query-understanding"))
    );
    assert_eq!(
        selected_evidence.get("evidenceKind"),
        Some(&serde_json::json!("search_strategy_flow_authority"))
    );
    assert_eq!(
        selected_evidence.get("selected"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        selected_evidence.get("plannerMaterialized"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        selected_evidence.get("retrievalRouteCount"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        selected_evidence.get("routePlanned"),
        Some(&serde_json::json!(true))
    );
    assert!(
        selected_evidence
            .get("proofTags")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("proofTags must be an array"))
            .contains(&serde_json::json!("route_planned"))
    );

    let blocked_evidence = projected_rows
        .iter()
        .find(|row| {
            row.get("candidateId")
                == Some(&serde_json::json!(
                    "docs/90_validation/90.01_validation.md#promotion-boundary"
                ))
        })
        .unwrap_or_else(|| panic!("blocked projected evidence row"));
    assert_eq!(
        blocked_evidence.get("blocked"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        blocked_evidence.get("selected"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        blocked_evidence.get("routePlanned"),
        Some(&serde_json::json!(false))
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "JSON bridge fixture is intentionally explicit"
)]
fn search_strategy_flow_rust_bridge_requires_section_granularity() {
    let trace = serde_json::json!({
        "intent": "find SearchStrategyFlow precision pruning",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "action": "keep",
                "reason": "file-level candidate should not materialize",
                "finalScore": 0.93,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            },
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "action": "keep",
                "reason": "section-level candidate should materialize",
                "finalScore": 0.91,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 800,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "rank": 1,
                "selected": true,
                "finalScore": 0.93,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            },
            {
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "rank": 2,
                "selected": true,
                "finalScore": 0.91,
                "action": "keep",
                "contextBudget": 800,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.93,
                "contextBudget": 1000,
                "reason": "file_level"
            },
            {
                "actionKind": "materialize",
                "candidateId": "docs/30_search_strategy/30.02_precision_pruning.md#precision-score",
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.91,
                "contextBudget": 800,
                "reason": "section_level"
            }
        ],
        "summary": {},
        "validation": {}
    });

    let enriched = enrich_wendaograph_search_strategy_flow_retrieval_routes(&trace.to_string())
        .unwrap_or_else(|error| panic!("enrich SearchStrategyFlow bridge trace: {error}"));
    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse enriched SearchStrategyFlow bridge trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));

    assert_eq!(routes.len(), 1);
    assert_eq!(
        routes[0].get("candidateId"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.02_precision_pruning.md#precision-score"
        ))
    );
    assert_eq!(
        routes[0].get("headingAnchor"),
        Some(&serde_json::json!("precision-score"))
    );
    assert_eq!(
        routes[0].get("directFileReadAllowed"),
        Some(&serde_json::json!(false))
    );
}
