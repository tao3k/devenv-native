use super::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, GRAPH_NEIGHBORS_ROUTE,
    SearchStrategyFlowFakeFlightScenario, SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    spawn_fake_search_strategy_flow_flight_service_with_graph_node_allowlist,
    spawn_fake_search_strategy_flow_flight_service_without_page_index,
};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight structured-code relation substitute fixture keeps the route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_classifies_code_relation_substitute() {
    let scenario = SearchStrategyFlowFakeFlightScenario::rust_reference();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find rust ppr search strategy",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": candidate_id.clone(),
                "action": "keep",
                "reason": "score",
                "finalScore": 0.92,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": candidate_id.clone(),
                "rank": 1,
                "selected": true,
                "finalScore": 0.92,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": candidate_id.clone(),
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.92,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let (base_url, server) =
        spawn_fake_search_strategy_flow_flight_service_with_graph_node_allowlist(scenario, &[])
            .await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "main")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| {
            panic!("execute missing graph-node materialization without aborting: {error}")
        });
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let route = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .and_then(|routes| routes.first())
        .unwrap_or_else(|| panic!("materialized route should exist"));

    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("graphMaterializationStatus"),
        Some(&serde_json::json!("structured-code-relation-substitute"))
    );
    assert!(
        route.get("resolvedGraphNodeId").is_none(),
        "structured-code relation substitute must not export a resolved graph-node id"
    );
    assert!(
        route
            .get("graphMaterializationWarning")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|warning| warning.contains("graph node")),
        "missing graph node must keep an explicit warning"
    );
    let graph_payload = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|receipt| {
            receipt.get("route").and_then(serde_json::Value::as_str) == Some(GRAPH_NEIGHBORS_ROUTE)
        })
        .unwrap_or_else(|| panic!("graph decoded payload receipt should be present"));
    assert_eq!(
        graph_payload.get("evidenceAnchor"),
        Some(&serde_json::json!(
            "structured-code-relation:node:ppr-runtime-search-strategy"
        ))
    );
    let graph_receipt = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|receipt| {
            receipt.get("route").and_then(serde_json::Value::as_str) == Some(GRAPH_NEIGHBORS_ROUTE)
        })
        .unwrap_or_else(|| panic!("graph route receipt should be present"));
    assert_eq!(graph_receipt.get("rowCount"), Some(&serde_json::json!(0)));
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight code substitute fixture keeps the route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_keeps_code_relation_when_page_index_missing() {
    let scenario = SearchStrategyFlowFakeFlightScenario::rust_reference();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find rust ppr search strategy",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "candidateInputSource": "rust-code-intelligence-inventory",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [
            {
                "candidateId": candidate_id.clone(),
                "action": "keep",
                "reason": "score",
                "finalScore": 0.92,
                "evidenceCoverage": 0.98,
                "graphScore": 0.95,
                "authorityScore": 0.93,
                "semanticScore": 0.0,
                "structuralScore": 0.9,
                "contextCost": 1000,
                "blocked": false
            }
        ],
        "frontier": [
            {
                "candidateId": candidate_id.clone(),
                "rank": 1,
                "selected": true,
                "finalScore": 0.92,
                "action": "keep",
                "contextBudget": 1000,
                "judgementKind": "graph_verified_candidate"
            }
        ],
        "plannerActions": [
            {
                "actionKind": "materialize",
                "candidateId": candidate_id.clone(),
                "targetCandidateId": "",
                "cycleAllowed": false,
                "requiresLlmJudgement": false,
                "score": 0.92,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let (base_url, server) =
        spawn_fake_search_strategy_flow_flight_service_without_page_index(scenario).await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "main")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| {
            panic!("execute code substitute without page-index materialization: {error}")
        });
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let route = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .and_then(|routes| routes.first())
        .unwrap_or_else(|| panic!("materialized route should exist"));

    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("graphMaterializationStatus"),
        Some(&serde_json::json!("structured-code-relation-substitute"))
    );
    assert!(
        route
            .get("pageIndexMaterializationWarning")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|warning| warning.contains("route class")),
        "code substitute should expose route-class page-index skipping"
    );
    let page_index_receipt = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|receipt| {
            receipt.get("route").and_then(serde_json::Value::as_str)
                == Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE)
        })
        .unwrap_or_else(|| panic!("page-index route receipt should be present"));
    assert_eq!(
        page_index_receipt.get("rowCount"),
        Some(&serde_json::json!(0))
    );
    assert_eq!(
        page_index_receipt.get("elapsedMs"),
        Some(&serde_json::json!(0))
    );
    let graph_payload = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|receipt| {
            receipt.get("route").and_then(serde_json::Value::as_str) == Some(GRAPH_NEIGHBORS_ROUTE)
        })
        .unwrap_or_else(|| panic!("graph decoded payload receipt should be present"));
    assert_eq!(
        graph_payload.get("evidenceAnchor"),
        Some(&serde_json::json!(
            "structured-code-relation:node:ppr-runtime-search-strategy"
        ))
    );
    assert_eq!(route.get("materializedRows"), Some(&serde_json::json!(1)));
}
