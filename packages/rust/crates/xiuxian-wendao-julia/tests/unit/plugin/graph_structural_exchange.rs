use std::sync::Arc;

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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
            id: "julia".to_string(),
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
