//! Process-managed `WendaoSearch` example services used by integration tests.

use std::env;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use super::service_runtime::{
    JuliaExampleServiceGuard, repo_root, reserve_service_port,
    wait_for_service_ready_with_attempts, wendaocodeparser_julia_project,
    wendaosearch_julia_project, wendaosearch_parser_summary_contract, wendaosearch_script,
};
use crate::{
    GraphStructuralFilterRequestRow, build_graph_structural_filter_request_batch,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    fetch_modelica_ast_query_analysis_blocking_for_repository,
    fetch_modelica_parser_file_summary_blocking_for_repository, graph_structural_pair_candidate_id,
};

const MODELICA_PARSER_SUMMARY_READY_SOURCE_ID: &str = "Warmup/package.mo";
const MODELICA_PARSER_SUMMARY_READY_SOURCE: &str = r"
within Warmup;
package Warmup
  import Modelica.Units.SI;

  model Probe
    parameter SI.Time tau = 1;
  end Probe;
end Warmup;
";
const MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS: u64 = 60;
const WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV: &str = "WENDAOSEARCH_SOLVER_DEMO_BASE_URL";
const WENDAOSEARCH_SOLVER_DEMO_READY_TIMEOUT_SECS: u64 = 90;
const JULIA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &["julia_file_summary", "julia_root_summary"];
const MODELICA_PARSER_SUMMARY_ROUTE_NAMES: &[&str] =
    &["modelica_file_summary", "modelica_ast_query"];
const ALL_PARSER_SUMMARY_ROUTE_NAMES: &[&str] = &[
    "julia_file_summary",
    "julia_root_summary",
    "modelica_file_summary",
    "modelica_ast_query",
];

/// Result summary for one graph-structural release prewarm probe.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralPrewarmReport {
    /// Number of logical Flight route calls performed.
    pub route_count: usize,
    /// Total elapsed time for all logical prewarm route calls.
    pub elapsed: Duration,
    /// Stable tiny candidate id used by the solver-demo prewarm request.
    pub candidate_id: String,
}

/// Warm-path timing statistics for a graph-structural release gate.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralWarmPathStats {
    /// Number of measured warm-path samples.
    pub sample_count: usize,
    /// Minimum observed elapsed milliseconds.
    pub min_ms: f64,
    /// Median observed elapsed milliseconds.
    pub median_ms: f64,
    /// P95 observed elapsed milliseconds.
    pub p95_ms: f64,
    /// Maximum observed elapsed milliseconds.
    pub max_ms: f64,
    /// `max_ms / min_ms`, or `0.0` when the minimum is effectively zero.
    pub spread_ratio: f64,
}

/// Stability limits for the graph-structural release gate.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralStabilizationLimits {
    /// Sequential and concurrent warm samples to measure after release prewarm.
    pub sample_count: usize,
    /// Maximum allowed warm-path p95 in milliseconds.
    pub max_p95_ms: f64,
    /// Maximum allowed warm-path max latency in milliseconds.
    pub max_max_ms: f64,
    /// Maximum allowed warm-path spread ratio once latency reaches the
    /// meaningful tail budget.
    pub max_spread_ratio: f64,
    /// Initial in-flight budget when the warm path is stable.
    pub preferred_max_in_flight: usize,
    /// Initial in-flight budget when the warm path has tail instability.
    pub degraded_max_in_flight: usize,
}

/// Stability reason emitted by the graph-structural release gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WendaoSearchGraphStructuralStabilizationReason {
    /// Sequential and concurrent samples stayed within the configured tail
    /// budget.
    Stable,
    /// Sequential samples crossed the configured tail budget.
    SequentialExceeded,
    /// Concurrent samples crossed the configured tail budget.
    ConcurrentExceeded,
    /// Both sequential and concurrent samples crossed the configured tail
    /// budget.
    BothExceeded,
}

impl Default for WendaoSearchGraphStructuralStabilizationLimits {
    fn default() -> Self {
        Self {
            sample_count: 3,
            max_p95_ms: 150.0,
            max_max_ms: 250.0,
            max_spread_ratio: 16.0,
            preferred_max_in_flight: 4,
            degraded_max_in_flight: 1,
        }
    }
}

impl WendaoSearchGraphStructuralStabilizationLimits {
    /// Returns a copy with a bounded non-zero sample count.
    #[must_use]
    pub fn with_sample_count(mut self, sample_count: usize) -> Self {
        self.sample_count = sample_count.max(1);
        self
    }
}

/// Release-gate report for a graph-structural Julia pod.
#[derive(Clone, Debug, PartialEq)]
pub struct WendaoSearchGraphStructuralStabilizationReport {
    /// The all-route first release prewarm report.
    pub prewarm: WendaoSearchGraphStructuralPrewarmReport,
    /// Sequential warm-path stats after release prewarm.
    pub sequential: WendaoSearchGraphStructuralWarmPathStats,
    /// Concurrent warm-path stats after release prewarm.
    pub concurrent: WendaoSearchGraphStructuralWarmPathStats,
    /// Whether both warm paths passed the configured limits.
    pub stable: bool,
    /// Why this report selected the recommended admission budget.
    pub stability_reason: WendaoSearchGraphStructuralStabilizationReason,
    /// Recommended initial Rust admission budget for this Julia pod.
    pub recommended_max_in_flight: usize,
}

/// Spawns the official `WendaoSearch` structural-rerank example in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_structural_rerank_service()
-> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service("structural_rerank", "demo").await
}

/// Spawns the official `WendaoSearch` structural-rerank example in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_structural_rerank_service()
-> (String, JuliaExampleServiceGuard) {
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaExampleServiceGuard::external());
    }
    spawn_wendaosearch_service("structural_rerank", "solver_demo").await
}

/// Spawns the official same-port multi-route `WendaoSearch` example in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_multi_route_service() -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_multi_route_service("demo").await
}

/// Spawns the official same-port multi-route `WendaoSearch` example in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the example script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_multi_route_service()
-> (String, JuliaExampleServiceGuard) {
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaExampleServiceGuard::external());
    }
    spawn_wendaosearch_multi_route_service("solver_demo").await
}

/// Prewarms the solver-demo graph-structural route family through the real
/// Flight endpoint before releasing that endpoint to user-visible traffic.
///
/// The probe uses the same tiny two-node request shape for explicit rerank,
/// explicit filter, manifest-discovered rerank, and manifest-discovered filter.
/// It does not change schemas, routes, fallback policy, or Julia thread
/// scheduling.
///
/// # Errors
///
/// Returns an error string when the endpoint rejects a prewarm request or
/// returns an unexpected solver-demo response.
pub async fn prewarm_wendaosearch_solver_demo_graph_structural_routes(
    base_url: &str,
) -> Result<WendaoSearchGraphStructuralPrewarmReport, String> {
    let started_at = Instant::now();
    let candidate_id = graph_structural_pair_candidate_id("node-1", "node-2")
        .map_err(|error| format!("build solver-demo prewarm candidate id: {error}"))?;
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(base_url);
    let manifest_repository = graph_structural_manifest_repository(base_url);

    prewarm_solver_demo_rerank_route(
        &explicit_rerank_repository,
        &candidate_id,
        "explicit structural-rerank",
    )
    .await?;
    prewarm_solver_demo_filter_route(
        &explicit_filter_repository,
        &candidate_id,
        "explicit constraint-filter",
    )
    .await?;
    prewarm_solver_demo_rerank_route(
        &manifest_repository,
        &candidate_id,
        "manifest structural-rerank",
    )
    .await?;
    prewarm_solver_demo_filter_route(
        &manifest_repository,
        &candidate_id,
        "manifest constraint-filter",
    )
    .await?;

    Ok(WendaoSearchGraphStructuralPrewarmReport {
        route_count: 4,
        elapsed: started_at.elapsed(),
        candidate_id,
    })
}

/// Prewarms and samples the solver-demo graph-structural route family before
/// releasing one Julia pod to user-visible traffic.
///
/// This helper keeps the route/schema/fallback contract unchanged. It gives
/// the Rust admission layer a stability-aware `max_in_flight` recommendation
/// rather than exposing first-route or tail-latency jitter to callers.
///
/// # Errors
///
/// Returns an error string when any release-prewarm or warm-sample route call
/// fails or returns an unexpected solver-demo response.
pub async fn stabilize_wendaosearch_solver_demo_graph_structural_routes(
    base_url: &str,
    limits: WendaoSearchGraphStructuralStabilizationLimits,
) -> Result<WendaoSearchGraphStructuralStabilizationReport, String> {
    let prewarm = prewarm_wendaosearch_solver_demo_graph_structural_routes(base_url).await?;
    let candidate_id = prewarm.candidate_id.clone();
    let explicit_rerank_repository = graph_structural_explicit_rerank_repository(base_url);
    let explicit_filter_repository = graph_structural_explicit_filter_repository(base_url);
    let manifest_repository = graph_structural_manifest_repository(base_url);

    let mut sequential_samples = Vec::with_capacity(limits.sample_count);
    for _ in 0..limits.sample_count {
        let started_at = Instant::now();
        prewarm_solver_demo_rerank_route(
            &explicit_rerank_repository,
            &candidate_id,
            "sequential structural-rerank",
        )
        .await?;
        prewarm_solver_demo_filter_route(
            &explicit_filter_repository,
            &candidate_id,
            "sequential constraint-filter",
        )
        .await?;
        prewarm_solver_demo_rerank_route(
            &manifest_repository,
            &candidate_id,
            "sequential manifest structural-rerank",
        )
        .await?;
        prewarm_solver_demo_filter_route(
            &manifest_repository,
            &candidate_id,
            "sequential manifest constraint-filter",
        )
        .await?;
        sequential_samples.push(started_at.elapsed());
    }

    let mut concurrent_samples = Vec::with_capacity(limits.sample_count);
    for _ in 0..limits.sample_count {
        let started_at = Instant::now();
        tokio::try_join!(
            prewarm_solver_demo_rerank_route(
                &explicit_rerank_repository,
                &candidate_id,
                "concurrent structural-rerank",
            ),
            prewarm_solver_demo_filter_route(
                &explicit_filter_repository,
                &candidate_id,
                "concurrent constraint-filter",
            ),
            prewarm_solver_demo_rerank_route(
                &manifest_repository,
                &candidate_id,
                "concurrent manifest structural-rerank",
            ),
            prewarm_solver_demo_filter_route(
                &manifest_repository,
                &candidate_id,
                "concurrent manifest constraint-filter",
            ),
        )?;
        concurrent_samples.push(started_at.elapsed());
    }

    let sequential = warm_path_stats_from_samples(&sequential_samples);
    let concurrent = warm_path_stats_from_samples(&concurrent_samples);
    let sequential_passes = warm_path_passes_limits(&sequential, &limits);
    let concurrent_passes = warm_path_passes_limits(&concurrent, &limits);
    let stability_reason = match (sequential_passes, concurrent_passes) {
        (true, true) => WendaoSearchGraphStructuralStabilizationReason::Stable,
        (false, true) => WendaoSearchGraphStructuralStabilizationReason::SequentialExceeded,
        (true, false) => WendaoSearchGraphStructuralStabilizationReason::ConcurrentExceeded,
        (false, false) => WendaoSearchGraphStructuralStabilizationReason::BothExceeded,
    };
    let stable = stability_reason == WendaoSearchGraphStructuralStabilizationReason::Stable;
    let recommended_max_in_flight = if stable {
        limits.preferred_max_in_flight.max(1)
    } else {
        limits
            .degraded_max_in_flight
            .max(1)
            .min(limits.preferred_max_in_flight.max(1))
    };

    Ok(WendaoSearchGraphStructuralStabilizationReport {
        prewarm,
        sequential,
        concurrent,
        stable,
        stability_reason,
        recommended_max_in_flight,
    })
}

/// Spawns the official `WendaoSearch` parser-summary service with the native
/// summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service() -> (String, JuliaExampleServiceGuard)
{
    spawn_wendaosearch_julia_parser_summary_service_with_attempts(1500).await
}

/// Spawns the official `WendaoSearch` parser-summary service with one explicit
/// readiness
/// attempt budget.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_julia_parser_summary_service_with_attempts(
    ready_attempts: usize,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_parser_summary_service(ready_attempts, JULIA_PARSER_SUMMARY_ROUTE_NAMES)
        .await
}

/// Spawns the official `WendaoSearch` parser-summary service with all native
/// Julia and Modelica summary routes mounted on the shared Flight endpoint.
///
/// # Panics
///
/// Panics when the service script cannot be resolved, the service fails to
/// start, or the Modelica parser-summary route readiness probe fails.
pub async fn spawn_wendaosearch_all_parser_summary_service() -> (String, JuliaExampleServiceGuard) {
    let (base_url, mut guard) =
        spawn_wendaosearch_parser_summary_service(1500, ALL_PARSER_SUMMARY_ROUTE_NAMES).await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch all-routes parser-summary readiness: {error}");
        },
    );
    (base_url, guard)
}

/// Spawns the official `WendaoSearch` parser-summary service for the Modelica
/// summary route.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_modelica_parser_summary_service()
-> (String, JuliaExampleServiceGuard) {
    let (base_url, mut guard) =
        spawn_wendaosearch_parser_summary_service(1500, MODELICA_PARSER_SUMMARY_ROUTE_NAMES).await;
    probe_wendaosearch_modelica_parser_summary_route_for_tests(base_url.as_str()).unwrap_or_else(
        |error| {
            guard.kill();
            panic!("wait for WendaoSearch Modelica parser-summary route readiness: {error}");
        },
    );
    (base_url, guard)
}

/// Probe the `WendaoSearch` Modelica parser-summary service on one explicit base
/// URL using the same file-summary plus ast-query warmup fixture as the linked
/// Julia integration helpers.
///
/// # Errors
///
/// Returns an error string when the fixed service does not accept the warmup
/// Modelica file-summary and ast-query requests on the configured Flight
/// endpoint.
pub fn probe_wendaosearch_modelica_parser_summary_route_for_tests(
    base_url: &str,
) -> Result<(), String> {
    wait_for_modelica_parser_summary_route_ready(base_url)
}

fn project_julia_command() -> Command {
    if executable_on_path("direnv") {
        let mut command = Command::new("direnv");
        command.arg("exec").arg(".").arg("julia");
        return command;
    }
    Command::new("julia")
}

fn executable_on_path(name: &str) -> bool {
    env::var_os("PATH")
        .is_some_and(|paths| env::split_paths(&paths).any(|path| path.join(name).is_file()))
}

fn configured_solver_demo_base_url() -> Option<String> {
    env::var(WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV)
        .ok()
        .filter(|value| !value.is_empty())
}

async fn wait_for_external_solver_demo_service(base_url: &str) {
    wait_for_service_ready_with_attempts(base_url, 600)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "wait for externally managed WendaoSearch solver-demo service readiness: {error}"
            )
        });
}

fn warm_path_stats_from_samples(samples: &[Duration]) -> WendaoSearchGraphStructuralWarmPathStats {
    let mut elapsed_values: Vec<f64> = samples
        .iter()
        .map(|sample| sample.as_secs_f64() * 1000.0)
        .collect();
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
    WendaoSearchGraphStructuralWarmPathStats {
        sample_count: elapsed_values.len(),
        min_ms,
        median_ms,
        p95_ms,
        max_ms,
        spread_ratio,
    }
}

fn percentile_from_sorted_values(sorted_values: &[f64], percentile_per_mille: usize) -> f64 {
    let last_index = sorted_values.len() - 1;
    let index = (last_index * percentile_per_mille).div_ceil(1000);
    sorted_values[index]
}

fn warm_path_passes_limits(
    stats: &WendaoSearchGraphStructuralWarmPathStats,
    limits: &WendaoSearchGraphStructuralStabilizationLimits,
) -> bool {
    if stats.p95_ms > limits.max_p95_ms || stats.max_ms > limits.max_max_ms {
        return false;
    }

    // A high spread ratio on tiny millisecond samples is not user-visible by
    // itself. Treat spread as a secondary gate only after max latency enters
    // the p95 budget region.
    stats.max_ms < limits.max_p95_ms || stats.spread_ratio <= limits.max_spread_ratio
}

#[cfg(test)]
mod tests {
    use super::{
        WendaoSearchGraphStructuralStabilizationLimits, WendaoSearchGraphStructuralWarmPathStats,
        warm_path_passes_limits,
    };

    fn stats(
        p95_ms: f64,
        max_ms: f64,
        spread_ratio: f64,
    ) -> WendaoSearchGraphStructuralWarmPathStats {
        WendaoSearchGraphStructuralWarmPathStats {
            sample_count: 3,
            min_ms: 1.0,
            median_ms: p95_ms,
            p95_ms,
            max_ms,
            spread_ratio,
        }
    }

    #[test]
    fn low_millisecond_spread_is_observed_without_degrading_admission() {
        let limits = WendaoSearchGraphStructuralStabilizationLimits {
            max_spread_ratio: 2.0,
            ..WendaoSearchGraphStructuralStabilizationLimits::default()
        };

        assert!(warm_path_passes_limits(&stats(44.0, 44.0, 44.0), &limits));
    }

    #[test]
    fn p95_or_max_budget_overflow_degrades_admission() {
        let limits = WendaoSearchGraphStructuralStabilizationLimits::default();

        assert!(!warm_path_passes_limits(&stats(151.0, 151.0, 1.0), &limits));
        assert!(!warm_path_passes_limits(&stats(149.0, 251.0, 1.0), &limits));
    }

    #[test]
    fn spread_ratio_is_secondary_only_inside_tail_budget_region() {
        let limits = WendaoSearchGraphStructuralStabilizationLimits {
            max_spread_ratio: 2.0,
            ..WendaoSearchGraphStructuralStabilizationLimits::default()
        };

        assert!(warm_path_passes_limits(&stats(12.0, 12.0, 12.0), &limits));
        assert!(!warm_path_passes_limits(&stats(149.0, 160.0, 3.0), &limits));
    }
}

async fn prewarm_solver_demo_rerank_route(
    repository: &RegisteredRepository,
    candidate_id: &str,
    route_label: &str,
) -> Result<(), String> {
    let rows =
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
        )
        .await
        .map_err(|error| format!("prewarm WendaoSearch {route_label}: {error}"))?;
    let row = rows.get(candidate_id).ok_or_else(|| {
        format!("prewarm WendaoSearch {route_label} returned no candidate `{candidate_id}`")
    })?;
    if !row.feasible {
        return Err(format!(
            "prewarm WendaoSearch {route_label} returned infeasible candidate `{candidate_id}`"
        ));
    }
    if row.pin_assignment != ["node-1".to_string()] {
        return Err(format!(
            "prewarm WendaoSearch {route_label} returned unexpected pin assignment {:?}",
            row.pin_assignment
        ));
    }
    Ok(())
}

async fn prewarm_solver_demo_filter_route(
    repository: &RegisteredRepository,
    candidate_id: &str,
    route_label: &str,
) -> Result<(), String> {
    let batch = build_graph_structural_filter_request_batch(&[GraphStructuralFilterRequestRow {
        query_id: "query-live".to_string(),
        candidate_id: candidate_id.to_string(),
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
    .map_err(|error| format!("build WendaoSearch {route_label} prewarm request: {error}"))?;
    let rows = fetch_graph_structural_filter_rows_for_repository(repository, &[batch])
        .await
        .map_err(|error| format!("prewarm WendaoSearch {route_label}: {error}"))?;
    let row = rows.get(candidate_id).ok_or_else(|| {
        format!("prewarm WendaoSearch {route_label} returned no candidate `{candidate_id}`")
    })?;
    if !row.accepted {
        return Err(format!(
            "prewarm WendaoSearch {route_label} rejected candidate `{candidate_id}`: {}",
            row.rejection_reason
        ));
    }
    if row.pin_assignment != ["node-1".to_string()] {
        return Err(format!(
            "prewarm WendaoSearch {route_label} returned unexpected pin assignment {:?}",
            row.pin_assignment
        ));
    }
    Ok(())
}

fn graph_structural_explicit_rerank_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "solver-demo-prewarm".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: json!({
                "graph_structural_transport": {
                    "base_url": base_url,
                    "structural_rerank": {
                        "route": "/graph/structural/rerank",
                        "schema_version": "v0-draft",
                        "timeout_secs": WENDAOSEARCH_SOLVER_DEMO_READY_TIMEOUT_SECS,
                    }
                }
            }),
        }],
    }
}

fn graph_structural_explicit_filter_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "solver-demo-prewarm".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: json!({
                "graph_structural_transport": {
                    "base_url": base_url,
                    "constraint_filter": {
                        "route": "/graph/structural/filter",
                        "schema_version": "v0-draft",
                        "timeout_secs": WENDAOSEARCH_SOLVER_DEMO_READY_TIMEOUT_SECS,
                    }
                }
            }),
        }],
    }
}

fn graph_structural_manifest_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "solver-demo-prewarm".to_string(),
        path: None,
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Config {
            id: "julia".to_string(),
            options: json!({
                "capability_manifest_transport": {
                    "base_url": base_url,
                    "route": "/plugin/capabilities",
                    "schema_version": "v0-draft",
                    "timeout_secs": WENDAOSEARCH_SOLVER_DEMO_READY_TIMEOUT_SECS,
                }
            }),
        }],
    }
}

async fn spawn_wendaosearch_service(
    route_name: &str,
    mode: &str,
) -> (String, JuliaExampleServiceGuard) {
    spawn_wendaosearch_service_with_code_parser_routes(route_name, mode, &[], 600).await
}

async fn spawn_wendaosearch_parser_summary_service(
    ready_attempts: usize,
    code_parser_route_names: &[&str],
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let contract = wendaosearch_parser_summary_contract();
    let base_url = format!("http://{}:{port}", contract.service.host);
    let script = contract.script_path();
    let child = project_julia_command()
        .arg(format!(
            "--project={}",
            wendaocodeparser_julia_project().display()
        ))
        .arg(script)
        .arg("--host")
        .arg(&contract.service.host)
        .arg("--port")
        .arg(port.to_string())
        .arg("--code-parser-route-names")
        .arg(code_parser_route_names.join(","))
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env(
            "JULIA_NUM_THREADS",
            env::var("WENDAOSEARCH_JULIA_NUM_THREADS").unwrap_or_else(|_| "8".to_string()),
        )
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("spawn real WendaoSearch parser-summary service: {error}"));
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch parser-summary service readiness: {error}");
        });

    (base_url, guard)
}

fn wait_for_modelica_parser_summary_route_ready(base_url: &str) -> Result<(), String> {
    let repository = modelica_parser_summary_ready_repository(base_url);
    let summary = fetch_modelica_parser_file_summary_blocking_for_repository(
        &repository,
        MODELICA_PARSER_SUMMARY_READY_SOURCE_ID,
        MODELICA_PARSER_SUMMARY_READY_SOURCE,
    )
    .map_err(|error| {
        format!("Modelica file-summary readiness probe failed for `{base_url}`: {error}")
    })?;
    if summary.class_name.as_deref() != Some("Warmup") {
        return Err(format!(
            "Modelica file-summary readiness probe returned unexpected class_name {:?} for `{base_url}`",
            summary.class_name
        ));
    }

    let analysis = fetch_modelica_ast_query_analysis_blocking_for_repository(
        &repository,
        MODELICA_PARSER_SUMMARY_READY_SOURCE_ID.into(),
        MODELICA_PARSER_SUMMARY_READY_SOURCE,
    )
    .map_err(|error| {
        format!("Modelica ast-query readiness probe failed for `{base_url}`: {error}")
    })?;
    if !analysis
        .modules
        .iter()
        .any(|module| module.qualified_name == "Warmup")
    {
        return Err(format!(
            "Modelica ast-query readiness probe returned no Warmup module for `{base_url}`"
        ));
    }

    Ok(())
}

fn modelica_parser_summary_ready_repository(base_url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: "linked-modelica-ready".to_string(),
        plugins: vec![RepositoryPluginConfig::Config {
            id: "modelica".to_string(),
            options: json!({
                "parser_summary_transport": {
                    "base_url": base_url,
                    "file_summary": {
                        "timeout_secs": MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS,
                    },
                    "ast_query": {
                        "timeout_secs": MODELICA_PARSER_SUMMARY_READY_TIMEOUT_SECS,
                    }
                }
            }),
        }],
        ..RegisteredRepository::default()
    }
}

async fn spawn_wendaosearch_service_with_code_parser_routes(
    route_name: &str,
    mode: &str,
    code_parser_route_names: &[&str],
    ready_attempts: usize,
) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let script = wendaosearch_script("run_search_service.jl");
    let mut command = project_julia_command();
    command
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(script)
        .arg("--route-name")
        .arg(route_name)
        .arg("--mode")
        .arg(mode)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if !code_parser_route_names.is_empty() {
        command
            .arg("--code-parser-route-names")
            .arg(code_parser_route_names.join(","));
    }
    let child = command.spawn().unwrap_or_else(|error| {
        panic!("spawn real WendaoSearch `{route_name}` `{mode}` service: {error}")
    });
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch service readiness: {error}");
        });

    (base_url, guard)
}

async fn spawn_wendaosearch_multi_route_service(mode: &str) -> (String, JuliaExampleServiceGuard) {
    let port = reserve_service_port();
    let base_url = format!("http://127.0.0.1:{port}");
    let script = wendaosearch_script("run_search_service.jl");
    let child = project_julia_command()
        .arg(format!(
            "--project={}",
            wendaosearch_julia_project().display()
        ))
        .arg(script)
        .arg("--route-names")
        .arg("capability_manifest,structural_rerank,constraint_filter")
        .arg("--mode")
        .arg(mode)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .current_dir(repo_root())
        .env("JULIA_LOAD_PATH", "@:@stdlib")
        .env("WENDAO_SEARCH_USE_ACTIVE_PROJECT", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("spawn real WendaoSearch multi-route `{mode}` service: {error}")
        });
    let mut guard = JuliaExampleServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), 600)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch multi-route service readiness: {error}");
        });

    (base_url, guard)
}
