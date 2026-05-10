use std::{
    env,
    path::{Path, PathBuf},
};

use super::{
    RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV,
    SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    parse_search_strategy_flow_probe_action, run_wendaograph_search_strategy_flow_json,
    search_strategy_flow_probe_action_route,
};
use xiuxian_wendao_runtime::transport::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE,
};

#[test]
fn search_strategy_flow_rust_bridge_rejects_blank_intent_before_launch() {
    let error = match run_wendaograph_search_strategy_flow_json("   ", ".") {
        Ok(trace) => panic!("blank intent should fail before launching Julia, got {trace}"),
        Err(error) => error,
    };

    assert_eq!(error, "SearchStrategyFlow intent must not be blank");
}

#[test]
fn wendaograph_search_strategy_flow_live_replay_runs_local_markdown_families_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow live replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    assert_search_strategy_flow_live_replay_family(
        "local-doc-authority-boundary",
        "SearchStrategyFlow WorkingKnowledge memory layer promotion boundary",
        "docs/",
        search_root.as_path(),
    );
    assert_search_strategy_flow_live_replay_family(
        "semantic-doc-working-knowledge",
        "semantic graph execution graph authority invariant",
        "semantic/",
        search_root.as_path(),
    );
}

fn search_strategy_flow_live_replay_search_root() -> PathBuf {
    match env::var_os("PRJ_ROOT") {
        Some(root) => PathBuf::from(root),
        None => PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(4)
            .unwrap_or_else(|| panic!("resolve repository root from Cargo manifest"))
            .to_path_buf(),
    }
}

fn assert_search_strategy_flow_live_replay_family(
    family: &str,
    intent: &str,
    expected_source_prefix: &str,
    search_root: &Path,
) {
    let trace = run_wendaograph_search_strategy_flow_json(intent, search_root)
        .unwrap_or_else(|error| panic!("run live SearchStrategyFlow replay for {family}: {error}"));
    let trace: serde_json::Value = serde_json::from_str(&trace).unwrap_or_else(|error| {
        panic!("parse live SearchStrategyFlow replay for {family}: {error}")
    });
    assert_eq!(
        trace.get("candidateInputSource"),
        Some(&serde_json::json!("rust-markdown-headings")),
        "{family} must use the current local Markdown bridge"
    );
    assert!(
        trace
            .get("candidateInputCount")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "{family} must discover local Markdown candidates"
    );

    let routes = trace
        .get("retrievalRoutes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{family} retrievalRoutes must be an array"));
    let projected_rows = trace
        .get("rustProjectedEvidenceRows")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{family} rustProjectedEvidenceRows must be an array"));
    assert!(!routes.is_empty(), "{family} must plan retrieval routes");
    assert!(
        !projected_rows.is_empty(),
        "{family} must project Rust evidence rows"
    );
    assert!(
        routes.iter().any(|route| {
            route
                .get("sourcePath")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with(expected_source_prefix))
        }),
        "{family} must route at least one {expected_source_prefix} candidate"
    );
    assert!(
        projected_rows.iter().any(|row| {
            row.get("sourcePath")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| path.starts_with(expected_source_prefix))
        }),
        "{family} must project at least one {expected_source_prefix} evidence row"
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "JSON bridge fixture is intentionally explicit"
)]
fn search_strategy_flow_rust_bridge_adds_planned_retrieval_routes() {
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

    assert_eq!(routes.len(), 1);
    assert_eq!(projected_rows.len(), 2);
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
