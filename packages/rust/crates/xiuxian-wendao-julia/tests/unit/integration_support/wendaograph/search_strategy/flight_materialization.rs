use super::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE, SearchStrategyFlowFakeFlightScenario,
    SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    spawn_fake_search_strategy_flow_flight_service,
    spawn_fake_search_strategy_flow_flight_service_for,
    spawn_fake_search_strategy_flow_flight_service_with_empty_repo_search,
    spawn_fake_search_strategy_flow_flight_service_with_graph_node_allowlist,
};

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight materialization fixture documents every decoded receipt"
)]
async fn search_strategy_flow_flight_materialization_executes_and_decodes_route_receipts() {
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
    let (base_url, server) = spawn_fake_search_strategy_flow_flight_service().await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute fake Flight materialization: {error}"));
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    assert_eq!(routes.len(), 1);
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("decodedPayloadStatus"),
        Some(&serde_json::json!("decoded"))
    );
    assert_eq!(route.get("materializedRows"), Some(&serde_json::json!(4)));
    assert_eq!(
        route.get("resolvedNodeId"),
        Some(&serde_json::json!("node:stage-1-query-understanding"))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );

    let route_receipts = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("routeReceipts must be an array"));
    let decoded_receipts = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("decodedPayloadReceipts must be an array"));
    assert_eq!(route_receipts.len(), 4);
    assert_eq!(decoded_receipts.len(), 4);
    for expected_route in [
        REPO_SEARCH_ROUTE,
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        GRAPH_NEIGHBORS_ROUTE,
    ] {
        assert!(
            route_receipts.iter().any(|receipt| {
                receipt.get("route").and_then(serde_json::Value::as_str) == Some(expected_route)
                    && receipt.get("rowCount").and_then(serde_json::Value::as_u64) == Some(1)
                    && receipt
                        .get("elapsedMs")
                        .and_then(serde_json::Value::as_u64)
                        .is_some()
            }),
            "timed route receipt for {expected_route} should exist"
        );
        assert!(
            decoded_receipts.iter().any(|receipt| {
                receipt.get("route").and_then(serde_json::Value::as_str) == Some(expected_route)
                    && receipt.get("rowCount").and_then(serde_json::Value::as_u64) == Some(1)
            }),
            "decoded receipt for {expected_route} should exist"
        );
    }
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight fallback fixture keeps the full route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_uses_source_path_when_repo_search_has_no_rows()
{
    let scenario = SearchStrategyFlowFakeFlightScenario::markdown();
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
    let (base_url, server) =
        spawn_fake_search_strategy_flow_flight_service_with_empty_repo_search(scenario).await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute fake Flight materialization: {error}"));
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
        Some(&serde_json::json!("source-path-fallback"))
    );
    assert_eq!(
        route.get("resolvedPageId"),
        Some(&serde_json::json!(
            "repo:docs:projection:explanation:doc:repo:docs:doc:30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(
            "docs/30_search_strategy/30.01_search_strategy_flow.md"
        ))
    );
    let route_receipts = route
        .get("routeReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("routeReceipts must be an array"));
    assert!(route_receipts.iter().any(|receipt| {
        receipt.get("route").and_then(serde_json::Value::as_str) == Some(REPO_SEARCH_ROUTE)
            && receipt.get("rowCount").and_then(serde_json::Value::as_u64) == Some(0)
    }));
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight reference fixture documents the non-Markdown route contract"
)]
async fn search_strategy_flow_flight_materialization_executes_non_markdown_reference_route() {
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
    let (base_url, server) = spawn_fake_search_strategy_flow_flight_service_for(scenario).await;
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, "docs")
        .unwrap_or_else(|error| panic!("create fake Flight materialization config: {error}"));

    let enriched =
        enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization(
            &trace.to_string(),
            &config,
        )
        .await
        .unwrap_or_else(|error| panic!("execute fake Flight materialization: {error}"));
    server.abort();

    let enriched: serde_json::Value = serde_json::from_str(&enriched)
        .unwrap_or_else(|error| panic!("parse materialized trace: {error}"));
    let routes = enriched
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("retrievalRoutes must be an array"));
    assert_eq!(routes.len(), 1);
    let route = &routes[0];
    assert_eq!(
        route.get("materializationStatus"),
        Some(&serde_json::json!("executed"))
    );
    assert_eq!(
        route.get("decodedPayloadStatus"),
        Some(&serde_json::json!("decoded"))
    );
    assert_eq!(
        route.get("sourcePath"),
        Some(&serde_json::json!(scenario.source_path))
    );
    assert_eq!(
        route.get("resolvedPageId"),
        Some(&serde_json::json!(scenario.page_id))
    );
    assert!(
        route
            .get("resolvedPageId")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|page_id| page_id.contains(":projection:reference:")),
        "non-Markdown routes must use reference projection semantics"
    );
    assert_eq!(
        route.get("resolvedNodeId"),
        Some(&serde_json::json!(scenario.node_id))
    );
    assert_eq!(
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(format!("docs/{}", scenario.source_path)))
    );

    let decoded_receipts = route
        .get("decodedPayloadReceipts")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("decodedPayloadReceipts must be an array"));
    assert!(decoded_receipts.iter().any(|receipt| {
        receipt.get("route").and_then(serde_json::Value::as_str) == Some(REPO_SEARCH_ROUTE)
            && receipt
                .get("evidenceAnchor")
                .and_then(serde_json::Value::as_str)
                == Some(format!("path:{}", scenario.source_path).as_str())
    }));
    assert!(decoded_receipts.iter().any(|receipt| {
        receipt.get("route").and_then(serde_json::Value::as_str)
            == Some(ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE)
            && receipt
                .get("evidenceAnchor")
                .and_then(serde_json::Value::as_str)
                == Some(scenario.node_id)
    }));
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight regression fixture keeps the repo-prefixed route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_falls_back_to_repo_relative_graph_node_id() {
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
        spawn_fake_search_strategy_flow_flight_service_with_graph_node_allowlist(
            scenario,
            &[scenario.source_path],
        )
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
            panic!("execute repo-relative graph-node fallback materialization: {error}")
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
        route.get("resolvedGraphNodeId"),
        Some(&serde_json::json!(scenario.source_path))
    );
    assert_eq!(
        route.get("graphMaterializationStatus"),
        Some(&serde_json::json!("resolved"))
    );
}

#[tokio::test]
#[expect(
    clippy::too_many_lines,
    reason = "Flight missing-graph regression fixture keeps the route trace shape explicit"
)]
async fn search_strategy_flow_flight_materialization_reports_missing_graph_node_without_aborting() {
    let scenario = SearchStrategyFlowFakeFlightScenario::markdown();
    let candidate_id = format!("{}#{}", scenario.source_path, scenario.node_anchor);
    let trace = serde_json::json!({
        "intent": "find search strategy flow",
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
        Some(&serde_json::json!("missing"))
    );
    assert!(
        route.get("resolvedGraphNodeId").is_none(),
        "missing graph node must not export a resolved id"
    );
    assert!(
        route
            .get("graphMaterializationWarning")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|warning| warning.contains("graph node")),
        "missing graph node must keep an explicit warning"
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
