use super::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, SearchStrategyFlowFakeFlightScenario,
    SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    spawn_fake_search_strategy_flow_flight_service_without_page_index,
};

#[tokio::test]
async fn search_strategy_flow_flight_materialization_warns_when_projected_page_is_missing() {
    let scenario = SearchStrategyFlowFakeFlightScenario::markdown();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find markdown search strategy flow evidence",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "candidateInputSource": "wendao-gateway-retrieval",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
        "stageReceipts": [],
        "candidates": [{
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
        }],
        "frontier": [{
            "candidateId": candidate_id.clone(),
            "rank": 1,
            "selected": true,
            "finalScore": 0.92,
            "action": "keep",
            "contextBudget": 1000,
            "judgementKind": "graph_verified_candidate"
        }],
        "plannerActions": [{
            "actionKind": "materialize",
            "candidateId": candidate_id,
            "targetCandidateId": "",
            "cycleAllowed": false,
            "requiresLlmJudgement": false,
            "score": 0.92,
            "contextBudget": 1000,
            "reason": "graph_materialize_candidate"
        }],
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
        .unwrap_or_else(|error| panic!("missing projected page must not abort trace: {error}"));
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
        Some(&serde_json::json!("projected-page-missing"))
    );
    assert!(
        route
            .get("pageIndexMaterializationWarning")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|warning| warning.contains("projected page")),
        "missing page-index page should remain visible in route warning"
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
}
