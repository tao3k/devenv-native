use xiuxian_wendao_runtime::transport::WENDAO_ARROW_FLIGHT_DATA_PLANE;

use crate::integration_support::search_strategy_flow_flight::SearchStrategyFlowServiceResponse;
use crate::integration_support::wendaograph::SearchStrategyFlowTimingMeasurements;

#[test]
fn search_strategy_flow_service_trace_uses_candidate_route_buckets_for_required_evidence() {
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "AGENTS.md#158-debt-closure-at-discovery-when-a-warning-lint-failure-modularity",
        ),
        "authority"
    );
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "docs/standards/AUDITOR_CODEX.md#document",
        ),
        "authority"
    );
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "docs/testing/README.md#default-validation-path",
        ),
        "validation"
    );
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md#document",
        ),
        "validation"
    );
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "docs/rfcs/2026-05-04-polyglot-compute-orchestrator-audit.md#2-2-existing-julia-compute-boundary",
        ),
        "authority"
    );
    assert_eq!(
        super::super::super::super::service_trace::frontier_route_bucket(
            "docs/10_graph_compute/10.01_link_graph_compute.md#relation-context",
        ),
        "link_graph"
    );
}

#[test]
fn search_strategy_flow_service_trace_reports_managed_warm_flight_policy() {
    let policy =
        super::super::super::super::service_trace::search_strategy_flow_performance_policy_json();

    assert_eq!(
        policy
            .get("serviceLifecycle")
            .and_then(serde_json::Value::as_str),
        Some("managed-warm-julia-service")
    );
    assert_eq!(
        policy
            .get("currentDataPlane")
            .and_then(serde_json::Value::as_str),
        Some(WENDAO_ARROW_FLIGHT_DATA_PLANE)
    );
    assert_eq!(
        policy
            .get("payloadEncoding")
            .and_then(serde_json::Value::as_str),
        Some("arrow-ipc-stream-bundle")
    );
    assert_eq!(
        policy
            .get("rustControlsMaterialization")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        policy
            .get("juliaOwnsAlgorithmCompute")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        policy
            .get("rustEmbeddingJulia")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        policy
            .get("jlrsAllowed")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        policy
            .get("cDataTransportEnabled")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    let lanes = policy
        .get("primaryOptimizationLanes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("performance policy must include optimization lanes"));
    assert!(
        lanes
            .iter()
            .any(|lane| lane.as_str() == Some("rust-duckdb-candidate-narrowing"))
    );
    assert!(
        lanes
            .iter()
            .any(|lane| lane.as_str() == Some("warm-submit-benchmark-gate"))
    );
}

#[test]
fn search_strategy_flow_service_trace_reserves_timing_breakdown_slots() {
    let response = empty_response();
    let timing =
        super::super::super::super::service_trace::search_strategy_flow_timing_breakdown_json(
            &response,
            SearchStrategyFlowTimingMeasurements::default(),
        );

    assert_eq!(
        timing.get("schemaVersion").and_then(|value| value.as_str()),
        Some("xiuxian_wendao.graph.search_strategy_flow.timing_breakdown.v1")
    );
    assert_eq!(
        timing.get("measured").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        timing.get("reportedBy").and_then(|value| value.as_str()),
        Some("rust-bridge-trace-contract")
    );
    for field in [
        "coldStartMs",
        "warmSubmitMs",
        "materializationMs",
        "llmJudgeMs",
        "algorithmServiceMs",
        "candidateDiscoveryMs",
    ] {
        assert_eq!(
            timing.get(field),
            Some(&serde_json::Value::Null),
            "{field} should be a reserved null slot until measured by a benchmark runner"
        );
    }
    assert_eq!(
        timing
            .get("llmJudgementRequiredCount")
            .and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        timing
            .get("materializationMeasured")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        timing
            .get("llmJudgeMeasured")
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}

#[test]
fn search_strategy_flow_service_trace_records_measured_timing_segments() {
    let response = empty_response();
    let timing =
        super::super::super::super::service_trace::search_strategy_flow_timing_breakdown_json(
            &response,
            SearchStrategyFlowTimingMeasurements {
                candidate_discovery: Some(12.5),
                algorithm_service: Some(34.0),
                materialization: Some(5.25),
            },
        );

    assert_eq!(
        timing.get("measured").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        timing
            .get("candidateDiscoveryMs")
            .and_then(serde_json::Value::as_f64),
        Some(12.5)
    );
    assert_eq!(
        timing
            .get("algorithmServiceMs")
            .and_then(serde_json::Value::as_f64),
        Some(34.0)
    );
    assert_eq!(
        timing
            .get("materializationMs")
            .and_then(serde_json::Value::as_f64),
        Some(5.25)
    );
    assert_eq!(
        timing
            .get("materializationMeasured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[test]
fn search_strategy_flow_service_trace_can_backfill_materialization_timing() {
    let trace = serde_json::json!({
        "timingBreakdown": {
            "measured": true,
            "materializationMeasured": false,
            "materializationMs": null,
        }
    })
    .to_string();
    let updated =
        super::super::super::super::service_trace::search_strategy_flow_trace_with_materialization_timing(
            trace.as_str(),
            7.5,
        )
        .unwrap_or_else(|error| panic!("backfill materialization timing: {error}"));
    let updated = serde_json::from_str::<serde_json::Value>(&updated)
        .unwrap_or_else(|error| panic!("parse updated timing trace: {error}"));
    let timing = updated
        .get("timingBreakdown")
        .unwrap_or_else(|| panic!("updated trace must keep timingBreakdown"));

    assert_eq!(
        timing.get("measured").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        timing
            .get("materializationMeasured")
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        timing
            .get("materializationMs")
            .and_then(serde_json::Value::as_f64),
        Some(7.5)
    );
}

fn empty_response() -> SearchStrategyFlowServiceResponse {
    SearchStrategyFlowServiceResponse {
        candidates: Vec::new(),
        transition_count: 0,
        frontier: Vec::new(),
        planner_actions: Vec::new(),
    }
}
