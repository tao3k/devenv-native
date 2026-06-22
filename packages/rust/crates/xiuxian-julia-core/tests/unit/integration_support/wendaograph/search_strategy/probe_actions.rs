use super::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE, SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_search_strategy_flow_probe_action, search_strategy_flow_probe_action_route,
};

#[test]
fn search_strategy_flow_probe_actions_are_whitelisted() {
    assert_eq!(
        search_strategy_flow_probe_action_route("expand_neighbors"),
        Ok(Some(GRAPH_NEIGHBORS_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("expand_neighbors:docs-fixture/docs/search.md"),
        Ok(Some(GRAPH_NEIGHBORS_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("open_parent_child"),
        Ok(Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("compare_provenance"),
        Ok(Some(REPO_SEARCH_ROUTE))
    );
    assert_eq!(
        search_strategy_flow_probe_action_route("open_adjacent_sections"),
        Ok(Some(ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE))
    );
    assert_eq!(search_strategy_flow_probe_action_route("stop"), Ok(None));
    assert!(parse_search_strategy_flow_probe_action("open_full_file").is_err());
}

#[tokio::test]
async fn search_strategy_flow_rust_bridge_rejects_invalid_flight_endpoint_before_execution() {
    let trace = serde_json::json!({
        "intent": "find query understanding",
        "backend": "rust-wendao-julia",
        "controlPlane": "rust",
        "graphProject": "/tmp/WendaoGraph.jl",
        "searchRoot": "/tmp/WendaoGraph.jl",
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
    let config = SearchStrategyFlowFlightMaterializationConfig::new("not a url", "docs")
        .unwrap_or_else(|error| panic!("create Flight materialization config: {error}"));

    let error =
        match enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        {
            Ok(trace) => panic!(
                "invalid endpoint should reject before executed receipts are fabricated, got {trace}"
            ),
            Err(error) => error,
        };

    assert!(error.contains("create SearchStrategyFlow Flight endpoint"));
}
