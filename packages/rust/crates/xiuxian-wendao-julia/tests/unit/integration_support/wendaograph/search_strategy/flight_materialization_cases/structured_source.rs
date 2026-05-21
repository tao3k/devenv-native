use super::{
    REPO_SEARCH_ROUTE, SearchStrategyFlowFakeFlightScenario,
    SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    spawn_fake_search_strategy_flow_flight_service_for,
};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Structured source-path preflight fixture keeps the route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_skips_repo_search_for_structured_markdown_source_path()
 {
    let scenario = SearchStrategyFlowFakeFlightScenario::markdown();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find query understanding",
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
                "finalScore": 0.91,
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
                "finalScore": 0.91,
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
                "score": 0.91,
                "contextBudget": 1000,
                "reason": "graph_materialize_candidate"
            }
        ],
        "summary": {},
        "validation": {}
    });
    let (base_url, server) = spawn_fake_search_strategy_flow_flight_service_for(scenario).await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute structured source-path materialization: {error}"));
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
        route.get("repoSearchResolutionStatus"),
        Some(&serde_json::json!("structured-source-path"))
    );
    assert!(
        route
            .get("repoSearchResolutionWarning")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|warning| warning.contains("structured source-path")),
        "structured source-path skip must remain auditable"
    );
    assert_eq!(
        route.get("resolvedPageId"),
        Some(&serde_json::json!(
            "repo:docs:projection:explanation:doc:repo:docs:doc:30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(scenario.source_path))
    );
    assert_eq!(route.get("materializedRows"), Some(&serde_json::json!(3)));

    let repo_search_receipt = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .find(|receipt| {
            receipt.get("route").and_then(serde_json::Value::as_str) == Some(REPO_SEARCH_ROUTE)
        })
        .unwrap_or_else(|| panic!("repo-search route receipt should be present"));
    assert_eq!(
        repo_search_receipt.get("rowCount"),
        Some(&serde_json::json!(0))
    );
    assert_eq!(
        repo_search_receipt.get("elapsedMs"),
        Some(&serde_json::json!(0))
    );
}
