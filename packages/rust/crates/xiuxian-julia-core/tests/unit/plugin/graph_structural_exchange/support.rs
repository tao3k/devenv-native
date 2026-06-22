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
    GraphStructuralKeywordOverlapCandidateMetadataInput, GraphStructuralKeywordOverlapQueryInput,
    GraphStructuralKeywordOverlapRawCandidateInput, GRAPH_STRUCTURAL_ACCEPTED_COLUMN,
    GRAPH_STRUCTURAL_CANDIDATE_ID_COLUMN, GRAPH_STRUCTURAL_EXPLANATION_COLUMN,
    GRAPH_STRUCTURAL_FEASIBLE_COLUMN, GRAPH_STRUCTURAL_FINAL_SCORE_COLUMN,
    GRAPH_STRUCTURAL_PIN_ASSIGNMENT_COLUMN, GRAPH_STRUCTURAL_REJECTION_REASON_COLUMN,
    GRAPH_STRUCTURAL_STRUCTURAL_SCORE_COLUMN,
    graph_structural_pair_candidate_id,
    integration_support::{
        WendaoSearchGraphStructuralStabilizationLimits,
        stabilize_wendaosearch_solver_demo_graph_structural_routes,
    },
    julia_plugin_test_support::common::ResultTestExt,
    julia_plugin_test_support::wendaosearch_services::{
        LIVE_REQUEST_TIMEOUT_SECS, LIVE_SERVICE_STARTUP_TIMEOUT_SECS,
        RUN_PROCESS_MANAGED_WENDAOSEARCH_TEST_ENV, await_live_step,
        ensure_process_managed_wendaosearch_solver_demo_service,
        local_wendaosearch_package_available, process_managed_wendaosearch_solver_demo_base_url,
        process_managed_wendaosearch_test_enabled, reserve_real_service_port,
        solver_demo_multi_route_base_url_for_port, solver_demo_wendaosearch_service_available,
        spawn_real_wendaosearch_demo_multi_route_service,
        spawn_real_wendaosearch_solver_demo_multi_route_service,
        spawn_real_wendaosearch_solver_demo_multi_route_service_with_options,
        wait_for_service_ready_with_attempts,
    },
};

use super::{
    GraphStructuralFilterRequestRow, GraphStructuralFilterScoreRow, GraphStructuralRerankRequestRow,
    GraphStructuralRerankScoreRow, build_graph_structural_filter_request_batch,
    build_graph_structural_rerank_request_batch, decode_graph_structural_filter_score_rows,
    decode_graph_structural_rerank_score_rows, fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_graph_structural_rerank_rows_for_repository,
};

const RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST_ENV: &str =
    "RUN_WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_TEST";
const WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS_ENV: &str = "WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_RUNS";
const WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES_ENV: &str =
    "WENDAOSEARCH_GRAPH_STRUCTURAL_PERF_WARM_SAMPLES";

fn build_graph_structural_keyword_overlap_query_inputs(
    query_id: impl Into<String>,
    retrieval_layer: i32,
    query_max_layers: i32,
    keyword_anchors: Vec<String>,
    edge_constraint_kinds: Vec<String>,
) -> crate::GraphStructuralKeywordOverlapQueryInputs {
    crate::build_graph_structural_keyword_overlap_query_inputs(
        GraphStructuralKeywordOverlapQueryInput {
            query_id: query_id.into(),
            retrieval_layer,
            query_max_layers,
            keyword_anchors,
            edge_constraint_kinds,
        },
    )
}

fn build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
    left_id: impl Into<String>,
    right_id: impl Into<String>,
    edge_kinds: Vec<String>,
    left_tags: Vec<String>,
    right_tags: Vec<String>,
) -> crate::GraphStructuralKeywordOverlapCandidateMetadataInputs {
    crate::build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
        GraphStructuralKeywordOverlapCandidateMetadataInput {
            left_id: left_id.into(),
            right_id: right_id.into(),
            edge_kinds,
            left_tags,
            right_tags,
        },
    )
}

fn build_graph_structural_keyword_overlap_raw_candidate_inputs(
    metadata_inputs: crate::GraphStructuralKeywordOverlapCandidateMetadataInputs,
    semantic_score: f64,
    dependency_score: f64,
    keyword_match: bool,
) -> crate::GraphStructuralKeywordOverlapRawCandidateInputs {
    crate::build_graph_structural_keyword_overlap_raw_candidate_inputs(
        GraphStructuralKeywordOverlapRawCandidateInput {
            metadata_inputs,
            semantic_score,
            dependency_score,
            keyword_match,
        },
    )
}

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
        constraint_kind: "pin_assignment".into(),
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
        constraint_kind: "pin_assignment".into(),
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
