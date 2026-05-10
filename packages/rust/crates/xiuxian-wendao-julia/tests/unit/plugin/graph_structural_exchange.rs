use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use arrow::array::{
    BooleanArray, Float64Array, ListArray, ListBuilder, StringArray, StringBuilder,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use crate::{
    build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    graph_structural_pair_candidate_id,
    integration_support::{
        WendaoSearchGraphStructuralStabilizationLimits,
        stabilize_wendaosearch_solver_demo_graph_structural_routes,
    },
    julia_plugin_test_support::common::ResultTestExt,
    julia_plugin_test_support::official_examples::{
        LIVE_REQUEST_TIMEOUT_SECS, LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV, await_live_step,
        ensure_process_managed_wendaosearch_solver_demo_service,
        process_managed_wendaosearch_solver_demo_base_url,
        process_managed_wendaosearch_test_enabled, reserve_real_service_port,
        solver_demo_multi_route_base_url_for_port,
        spawn_real_wendaosearch_demo_multi_route_service,
        spawn_real_wendaosearch_solver_demo_multi_route_service,
        spawn_real_wendaosearch_solver_demo_multi_route_service_with_options,
        wait_for_service_ready_with_attempts,
    },
};

use super::{
    GRAPH_STRUCTURAL_ACCEPTED_COLUMN, GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, GRAPH_STRUCTURAL_QUERY_ID_COLUMN,
    GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN, GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN, GraphStructuralFilterRequestRow,
    GraphStructuralFilterScoreRow, GraphStructuralRerankRequestRow, GraphStructuralRerankScoreRow,
    build_graph_structural_filter_request_batch, build_graph_structural_rerank_request_batch,
    decode_graph_structural_filter_score_rows, decode_graph_structural_rerank_score_rows,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_graph_structural_rerank_rows_for_repository,
};

const RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV: &str =
    "RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST";
const WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS_ENV: &str = "WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS";
const WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES_ENV: &str =
    "WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES";

#[derive(Clone, Debug)]
struct LivePerfMeasurement {
    profile: &'static str,
    label: &'static str,
    logical_request_count: usize,
    run_index: usize,
    sample_index: usize,
    elapsed: Duration,
}

impl LivePerfMeasurement {
    fn elapsed_ms(&self) -> f64 {
        self.elapsed.as_secs_f64() * 1000.0
    }

    fn average_request_ms(&self) -> f64 {
        if self.logical_request_count == 0 {
            0.0
        } else {
            let request_count =
                u32::try_from(self.logical_request_count).map_or(f64::INFINITY, f64::from);
            self.elapsed_ms() / request_count
        }
    }
}

#[test]
fn build_graph_structural_rerank_request_batch_uses_contract_columns() {
    let batch = build_graph_structural_rerank_request_batch(&[GraphStructuralRerankRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 0,
        query_max_layers: 2,
        semantic_score: 0.7,
        dependency_score: 0.6,
        keyword_score: 0.4,
        tag_score: 0.3,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("rerank request batch");

    assert_eq!(
        batch.schema().field(0).name(),
        GRAPH_STRUCTURAL_QUERY_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(1).name(),
        GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN
    );
    assert_eq!(
        batch.schema().field(4).name(),
        GRAPH_STRUCTURAL_SEMANTIC_SCORE_COLUMN
    );
    assert_eq!(
        batch.schema().field(14).name(),
        GRAPH_STRUCTURAL_CANDIDATE_EDGE_KINDS_COLUMN
    );
}

#[test]
fn build_graph_structural_filter_request_batch_rejects_misaligned_anchors() {
    let error = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 1,
        query_max_layers: 3,
        constraint_kind: "boundary-match".to_string(),
        required_boundary_size: 2,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string(), "tag:core".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .err_or_panic("misaligned anchors must fail");

    assert!(
        error
            .to_string()
            .contains("anchor columns must stay aligned"),
        "unexpected error: {error}"
    );
}

#[test]
fn decode_graph_structural_rerank_score_rows_materializes_values() {
    let rows = decode_graph_structural_rerank_score_rows(&[rerank_response_batch()])
        .or_panic("rerank decode");

    assert_eq!(
        rows.get("candidate-a"),
        Some(&GraphStructuralRerankScoreRow {
            candidate_id: "candidate-a".to_string(),
            feasible: true,
            structural_score: 0.91,
            final_score: 0.87,
            pin_assignment: vec!["pin:entry".to_string(), "pin:exit".to_string()],
            explanation: "accepted".to_string(),
        })
    );
}

#[test]
fn decode_graph_structural_filter_score_rows_materializes_values() {
    let rows = decode_graph_structural_filter_score_rows(&[filter_response_batch()])
        .or_panic("filter decode");

    assert_eq!(
        rows.get("candidate-a"),
        Some(&GraphStructuralFilterScoreRow {
            candidate_id: "candidate-a".to_string(),
            accepted: false,
            structural_score: 0.52,
            pin_assignment: vec!["pin:entry".to_string()],
            rejection_reason: "missing boundary".to_string(),
        })
    );
}

#[tokio::test]
async fn fetch_graph_structural_rerank_rows_for_repository_rejects_missing_transport() {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({}),
        }],
    };

    let batch = build_graph_structural_rerank_request_batch(&[GraphStructuralRerankRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 0,
        query_max_layers: 2,
        semantic_score: 0.7,
        dependency_score: 0.6,
        keyword_score: 0.4,
        tag_score: 0.3,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("rerank request batch");

    let error = fetch_graph_structural_rerank_rows_for_repository(&repository, &[batch])
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_rejects_missing_transport()
 {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({}),
        }],
    };

    let error = fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository(
        &repository,
        &build_graph_structural_keyword_overlap_query_inputs(
            "query-1",
            0,
            2,
            vec!["alpha".to_string()],
            vec!["depends_on".to_string()],
        ),
        &[
            build_graph_structural_keyword_overlap_pair_candidate_inputs_from_raw(
                build_graph_structural_keyword_overlap_raw_candidate_inputs(
                    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                        "node-1",
                        "node-2",
                        vec!["depends_on".to_string()],
                        vec!["alpha".to_string(), "core".to_string()],
                        vec!["core".to_string()],
                    ),
                    0.7,
                    0.6,
                    true,
                ),
            ),
        ],
    )
    .await
    .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates_rejects_missing_transport()
 {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({}),
        }],
    };

    let error =
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            &repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-raw",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string()],
                ),
                0.7,
                0.6,
                true,
            )],
        )
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/rerank"),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn fetch_graph_structural_filter_rows_for_repository_rejects_missing_transport() {
    let repository = RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({}),
        }],
    };

    let batch = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-1".to_string(),
        candidate_id: "candidate-a".to_string(),
        retrieval_layer: 1,
        query_max_layers: 3,
        constraint_kind: "boundary-match".to_string(),
        required_boundary_size: 2,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["symbol:entry".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("filter request batch");

    let error = fetch_graph_structural_filter_rows_for_repository(&repository, &[batch])
        .await
        .err_or_panic("missing graph-structural transport must fail");
    assert!(
        error.to_string().contains("/graph/structural/filter"),
        "unexpected error: {error}"
    );
}

fn graph_structural_explicit_rerank_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": base_url,
                    "structural_rerank": {
                        "route": "/graph/structural/rerank",
                        "schema_version": "v0-draft",
                        "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                    }
                }
            }),
        }],
    }
}

async fn assert_demo_multi_route_rerank_rows(repository: &RegisteredRepository) {
    let rows = await_live_step(
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-live",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string()],
                ),
                0.7,
                0.6,
                true,
            )],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "real WendaoSearch graph-structural rerank",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("real WendaoSearch graph-structural rerank should succeed: {error}")
    });

    let candidate_id =
        graph_structural_pair_candidate_id("node-1", "node-2").or_panic("stable pair candidate id");
    let row = rows
        .get(&candidate_id)
        .unwrap_or_else(|| panic!("missing candidate `{candidate_id}` in live response"));
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.feasible);
    assert!((row.structural_score - 0.935).abs() < 1e-12);
    assert!((row.final_score - 1.035).abs() < 1e-12);
    assert_eq!(
        row.pin_assignment,
        vec!["node-1".to_string(), "node-2".to_string()]
    );
    assert_eq!(
        row.explanation,
        "demo feasible candidate with 2 nodes and 1 edge kinds"
    );
}

fn graph_structural_manifest_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "capability_manifest_transport": {
                    "base_url": base_url,
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft",
                    "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                }
            }),
        }],
    }
}

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
async fn fetch_graph_structural_demo_rerank_rows_for_repository_against_real_wendaosearch_multi_route_service()
 {
    let port = reserve_real_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let mut service = spawn_real_wendaosearch_demo_multi_route_service(port);
    let explicit_repository = graph_structural_explicit_rerank_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&format!("http://127.0.0.1:{port}"), 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch multi-route Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch multi-route Flight service: {error}")
    });

    assert_demo_multi_route_rerank_rows(&explicit_repository).await;
    assert_demo_multi_route_rerank_rows(&manifest_repository).await;
    service.kill();
}

async fn assert_solver_demo_explicit_rerank_rows(repository: &RegisteredRepository) {
    let rows = await_live_step(
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-live",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string()],
                ),
                0.7,
                0.6,
                true,
            )],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "real WendaoSearch solver-demo graph-structural rerank",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("real WendaoSearch solver-demo graph-structural rerank should succeed: {error}")
    });

    let candidate_id =
        graph_structural_pair_candidate_id("node-1", "node-2").or_panic("stable pair candidate id");
    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo explicit response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.feasible);
    assert!(row.structural_score > 0.0);
    assert!(row.final_score > row.structural_score);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert!(
        row.explanation
            .contains("solver_demo feasible candidate via rydberg solve"),
        "unexpected explanation: {}",
        row.explanation
    );
}

fn graph_structural_explicit_filter_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "demo".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia-code-parser".to_string(),
            options: serde_json::json!({
                "graph_structural_transport": {
                    "base_url": base_url,
                    "constraint_filter": {
                        "route": "/graph/structural/filter",
                        "schema_version": "v0-draft",
                        "timeout_secs": LIVE_REQUEST_TIMEOUT_SECS
                    }
                }
            }),
        }],
    }
}

async fn assert_solver_demo_multi_route_rerank_rows(repository: &RegisteredRepository) {
    let rows = await_live_step(
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                "query-live",
                0,
                2,
                vec!["alpha".to_string()],
                vec!["depends_on".to_string()],
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                    "node-1",
                    "node-2",
                    vec!["depends_on".to_string()],
                    vec!["alpha".to_string(), "core".to_string()],
                    vec!["core".to_string()],
                ),
                0.7,
                0.6,
                true,
            )],
        ),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo rerank",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("manifest-discovered real WendaoSearch solver-demo rerank should succeed: {error}")
    });

    let candidate_id =
        graph_structural_pair_candidate_id("node-1", "node-2").or_panic("stable pair candidate id");
    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo multi-route response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.feasible);
    assert!(row.structural_score > 0.0);
    assert!(row.final_score > row.structural_score);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert!(
        row.explanation
            .contains("solver_demo feasible candidate via rydberg solve"),
        "unexpected explanation: {}",
        row.explanation
    );
}

async fn assert_solver_demo_explicit_filter_rows(repository: &RegisteredRepository) {
    let candidate_id =
        graph_structural_pair_candidate_id("node-1", "node-2").or_panic("stable pair candidate id");
    let batch = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-live".to_string(),
        candidate_id: candidate_id.clone(),
        retrieval_layer: 0,
        query_max_layers: 2,
        constraint_kind: "pin_assignment".to_string(),
        required_boundary_size: 1,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["alpha".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("solver-demo filter request batch");

    let rows = await_live_step(
        fetch_graph_structural_filter_rows_for_repository(repository, &[batch]),
        LIVE_REQUEST_TIMEOUT_SECS,
        "real WendaoSearch solver-demo graph-structural filter",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("real WendaoSearch solver-demo graph-structural filter should succeed: {error}")
    });

    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo filter response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.accepted);
    assert!(row.structural_score > 0.0);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert_eq!(row.rejection_reason, "");
}

async fn assert_solver_demo_multi_route_filter_rows(repository: &RegisteredRepository) {
    let candidate_id =
        graph_structural_pair_candidate_id("node-1", "node-2").or_panic("stable pair candidate id");
    let batch = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-live".to_string(),
        candidate_id: candidate_id.clone(),
        retrieval_layer: 0,
        query_max_layers: 2,
        constraint_kind: "pin_assignment".to_string(),
        required_boundary_size: 1,
        anchor_planes: vec!["semantic".to_string()],
        anchor_values: vec!["alpha".to_string()],
        edge_constraint_kinds: vec!["depends_on".to_string()],
        candidate_node_ids: vec!["node-1".to_string(), "node-2".to_string()],
        candidate_edge_sources: vec!["node-1".to_string()],
        candidate_edge_destinations: vec!["node-2".to_string()],
        candidate_edge_kinds: vec!["depends_on".to_string()],
    }])
    .or_panic("solver-demo manifest filter request batch");

    let rows = await_live_step(
        fetch_graph_structural_filter_rows_for_repository(repository, &[batch]),
        LIVE_REQUEST_TIMEOUT_SECS,
        "manifest-discovered real WendaoSearch solver-demo filter",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("manifest-discovered real WendaoSearch solver-demo filter should succeed: {error}")
    });

    let row = rows.get(&candidate_id).unwrap_or_else(|| {
        panic!("missing candidate `{candidate_id}` in solver-demo manifest filter response")
    });
    assert_eq!(row.candidate_id, candidate_id);
    assert!(row.accepted);
    assert!(row.structural_score > 0.0);
    assert_eq!(row.pin_assignment, vec!["node-1".to_string()]);
    assert_eq!(row.rejection_reason, "");
}

fn print_live_perf_metric(measurement: &LivePerfMeasurement) {
    let elapsed_ms = measurement.elapsed_ms();
    assert!(
        elapsed_ms.is_finite(),
        "live graph perf metric `{}` elapsed time is not finite",
        measurement.label
    );
    println!(
        "wendaosearch_graph_structural_live_perf profile={} metric={} run={} sample={} logical_requests={} elapsed_ms={:.3} average_request_ms={:.3}",
        measurement.profile,
        measurement.label,
        measurement.run_index,
        measurement.sample_index,
        measurement.logical_request_count,
        elapsed_ms,
        measurement.average_request_ms(),
    );
}

async fn measure_live_perf_step<F>(
    profile: &'static str,
    label: &'static str,
    logical_request_count: usize,
    run_index: usize,
    sample_index: usize,
    future: F,
) -> LivePerfMeasurement
where
    F: std::future::Future<Output = ()>,
{
    let started_at = Instant::now();
    future.await;
    let measurement = LivePerfMeasurement {
        profile,
        label,
        logical_request_count,
        run_index,
        sample_index,
        elapsed: started_at.elapsed(),
    };
    print_live_perf_metric(&measurement);
    measurement
}

fn print_live_perf_summary(measurements: &[LivePerfMeasurement]) {
    let mut grouped: BTreeMap<(&str, &str), Vec<f64>> = BTreeMap::new();
    for measurement in measurements {
        grouped
            .entry((measurement.profile, measurement.label))
            .or_default()
            .push(measurement.elapsed_ms());
    }

    for ((profile, label), mut elapsed_values) in grouped {
        elapsed_values.sort_by(f64::total_cmp);
        let min_ms = elapsed_values[0];
        let median_ms = percentile_from_sorted_values(&elapsed_values, 500);
        let p95_ms = percentile_from_sorted_values(&elapsed_values, 950);
        let max_ms = elapsed_values[elapsed_values.len() - 1];
        let spread_ratio = if min_ms <= f64::EPSILON {
            0.0
        } else {
            max_ms / min_ms
        };
        println!(
            "wendaosearch_graph_structural_live_perf_summary profile={profile} metric={label} samples={} min_ms={min_ms:.3} median_ms={median_ms:.3} p95_ms={p95_ms:.3} max_ms={max_ms:.3} spread_ratio={spread_ratio:.3}",
            elapsed_values.len(),
        );
    }
}

fn percentile_from_sorted_values(sorted_values: &[f64], percentile_per_mille: usize) -> f64 {
    let last_index = sorted_values.len() - 1;
    let index = (last_index * percentile_per_mille).div_ceil(1000);
    sorted_values[index]
}

fn live_perf_env_usize(name: &str, default_value: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_value)
}

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
async fn fetch_graph_structural_solver_demo_rows_for_repository_via_manifest_discovery_against_real_wendaosearch_multi_route_service()
 {
    let port = reserve_real_service_port();
    let base_url = solver_demo_multi_route_base_url_for_port(port);
    let mut service = spawn_real_wendaosearch_solver_demo_multi_route_service(port);
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(&base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch solver-demo multi-route Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch solver-demo multi-route Flight service: {error}")
    });

    assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
    assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
    assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
    assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
    service.kill();
}

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
#[expect(
    clippy::large_futures,
    reason = "live perf proof is opt-in and route-complete"
)]
async fn graph_structural_live_perf_against_real_wendaosearch_solver_demo_multi_route_service() {
    if std::env::var_os(RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoSearch graph-structural live perf profile; set {RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV}=1"
        );
        return;
    }

    let run_count = live_perf_env_usize(WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS_ENV, 1);
    let warm_sample_count =
        live_perf_env_usize(WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES_ENV, 3);
    let mut measurements = Vec::new();
    for run_index in 0..run_count {
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "cold",
                false,
                None,
                false,
                run_index,
                warm_sample_count,
            )
            .await,
        );
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "prewarmed",
                true,
                Some("none"),
                false,
                run_index,
                warm_sample_count,
            )
            .await,
        );
        measurements.extend(
            measure_solver_demo_live_perf_profile(
                "prewarmed-flight-probe",
                true,
                Some("none"),
                true,
                run_index,
                warm_sample_count,
            )
            .await,
        );
    }
    print_live_perf_summary(&measurements);
}

#[expect(
    clippy::large_futures,
    clippy::too_many_lines,
    reason = "live perf proof keeps route scenarios together"
)]
async fn measure_solver_demo_live_perf_profile(
    profile: &'static str,
    warmup_on_start: bool,
    thread_pinning_policy: Option<&str>,
    prewarm_flight_routes: bool,
    run_index: usize,
    warm_sample_count: usize,
) -> Vec<LivePerfMeasurement> {
    let mut measurements = Vec::new();
    let port = reserve_real_service_port();
    let base_url = solver_demo_multi_route_base_url_for_port(port);
    let mut service = spawn_real_wendaosearch_solver_demo_multi_route_service_with_options(
        port,
        warmup_on_start,
        thread_pinning_policy,
    );
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(&base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    let startup_started_at = Instant::now();
    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for real WendaoSearch solver-demo live perf Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for real WendaoSearch solver-demo live perf Flight service: {error}")
    });
    let startup_measurement = LivePerfMeasurement {
        profile,
        label: "startup-wait",
        logical_request_count: 0,
        run_index,
        sample_index: 0,
        elapsed: startup_started_at.elapsed(),
    };
    print_live_perf_metric(&startup_measurement);
    measurements.push(startup_measurement);

    if prewarm_flight_routes {
        measurements.push(
            measure_live_perf_step(
                profile,
                "flight-release-gate",
                4 + warm_sample_count * 8,
                run_index,
                0,
                async {
                    prewarm_solver_demo_live_routes(&base_url, warm_sample_count).await;
                },
            )
            .await,
        );
    }

    measurements.push(
        measure_live_perf_step(profile, "first-explicit-rerank", 1, run_index, 0, async {
            assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
        })
        .await,
    );
    measurements.push(
        measure_live_perf_step(profile, "first-explicit-filter", 1, run_index, 0, async {
            assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
        })
        .await,
    );
    measurements.push(
        measure_live_perf_step(profile, "first-manifest-rerank", 1, run_index, 0, async {
            assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
        })
        .await,
    );
    measurements.push(
        measure_live_perf_step(profile, "first-manifest-filter", 1, run_index, 0, async {
            assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
        })
        .await,
    );

    for sample_index in 0..warm_sample_count {
        measurements.push(
            measure_live_perf_step(
                profile,
                "sequential-all-routes",
                4,
                run_index,
                sample_index,
                async {
                    assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
                    assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
                    assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
                    assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
                },
            )
            .await,
        );
    }

    for sample_index in 0..warm_sample_count {
        measurements.push(
            measure_live_perf_step(
                profile,
                "concurrent-all-routes",
                4,
                run_index,
                sample_index,
                async {
                    tokio::join!(
                        assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository),
                        assert_solver_demo_explicit_filter_rows(&explicit_filter_repository),
                        assert_solver_demo_multi_route_rerank_rows(&manifest_repository),
                        assert_solver_demo_multi_route_filter_rows(&manifest_repository),
                    );
                },
            )
            .await,
        );
    }

    service.kill();
    measurements
}

#[expect(
    clippy::large_futures,
    reason = "prewarm proof is opt-in live harness code"
)]
async fn prewarm_solver_demo_live_routes(base_url: &str, stabilization_sample_count: usize) {
    let report = stabilize_wendaosearch_solver_demo_graph_structural_routes(
        base_url,
        WendaoSearchGraphStructuralStabilizationLimits::default()
            .with_sample_count(stabilization_sample_count),
    )
    .await
    .unwrap_or_else(|error| panic!("stabilize WendaoSearch solver-demo routes: {error}"));
    assert_eq!(report.prewarm.route_count, 4);
    println!(
        "wendaosearch_graph_structural_release_gate stable={} reason={:?} recommended_max_in_flight={} sequential_p95_ms={:.3} sequential_max_ms={:.3} sequential_spread_ratio={:.3} concurrent_p95_ms={:.3} concurrent_max_ms={:.3} concurrent_spread_ratio={:.3}",
        report.stable,
        report.stability_reason,
        report.recommended_max_in_flight,
        report.sequential.p95_ms,
        report.sequential.max_ms,
        report.sequential.spread_ratio,
        report.concurrent.p95_ms,
        report.concurrent.max_ms,
        report.concurrent.spread_ratio,
    );
}

#[tokio::test]
#[serial_test::serial(wendaosearch_solver_demo_live)]
#[expect(clippy::large_futures, reason = "process-managed live proof is opt-in")]
async fn fetch_graph_structural_solver_demo_rows_for_repository_against_process_managed_wendaosearch_service()
 {
    if !process_managed_wendaosearch_test_enabled() {
        eprintln!(
            "skipping process-managed WendaoSearch live proof; set {RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV}=1"
        );
        return;
    }

    let _service = ensure_process_managed_wendaosearch_solver_demo_service()
        .await
        .or_panic("ensure process-managed WendaoSearch solver-demo Flight service");
    let base_url = process_managed_wendaosearch_solver_demo_base_url()
        .or_panic("resolve process-managed WendaoSearch solver-demo base URL");
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(&base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(&base_url);
    let manifest_repository = graph_structural_manifest_repository(&base_url);

    await_live_step(
        wait_for_service_ready_with_attempts(&base_url, 600),
        LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        "wait for process-managed WendaoSearch solver-demo Flight service",
    )
    .await
    .unwrap_or_else(|error| {
        panic!("wait for process-managed WendaoSearch solver-demo Flight service: {error}")
    });

    prewarm_solver_demo_live_routes(&base_url, 2).await;

    assert_solver_demo_explicit_rerank_rows(&explicit_rerank_repository).await;
    assert_solver_demo_explicit_filter_rows(&explicit_filter_repository).await;
    assert_solver_demo_multi_route_rerank_rows(&manifest_repository).await;
    assert_solver_demo_multi_route_filter_rows(&manifest_repository).await;
}

fn rerank_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, DataType::Utf8, false),
            Field::new(GRAPH_STRUCTURAL_FEASIBLE_COLUMN, DataType::Boolean, false),
            Field::new(
                GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
                DataType::Float64,
                false,
            ),
            Field::new(
                GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
                DataType::Float64,
                false,
            ),
            Field::new(
                GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                false,
            ),
            Field::new(GRAPH_STRUCTURAL_EXPLANATION_COLUMN, DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(BooleanArray::from(vec![true])),
            Arc::new(Float64Array::from(vec![0.91])),
            Arc::new(Float64Array::from(vec![0.87])),
            Arc::new(list_utf8_array(vec![vec!["pin:entry", "pin:exit"]])),
            Arc::new(StringArray::from(vec!["accepted"])),
        ],
    )
    .or_panic("rerank response batch")
}

fn filter_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new(GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, DataType::Utf8, false),
            Field::new(GRAPH_STRUCTURAL_ACCEPTED_COLUMN, DataType::Boolean, false),
            Field::new(
                GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
                DataType::Float64,
                false,
            ),
            Field::new(
                GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN,
                DataType::List(Arc::new(Field::new("item", DataType::Utf8, true))),
                false,
            ),
            Field::new(
                GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
                DataType::Utf8,
                false,
            ),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["candidate-a"])),
            Arc::new(BooleanArray::from(vec![false])),
            Arc::new(Float64Array::from(vec![0.52])),
            Arc::new(list_utf8_array(vec![vec!["pin:entry"]])),
            Arc::new(StringArray::from(vec!["missing boundary"])),
        ],
    )
    .or_panic("filter response batch")
}

fn list_utf8_array(values: Vec<Vec<&str>>) -> ListArray {
    let mut builder = ListBuilder::new(StringBuilder::new());
    for row in values {
        for value in row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}
