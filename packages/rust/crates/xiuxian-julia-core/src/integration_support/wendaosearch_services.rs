//! Process-managed `WendaoSearch` services used by integration tests.

use std::collections::HashSet;
use std::env;
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use serde_json::json;
use xiuxian_wendao_core::repo_intelligence::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy,
};

use super::{
    JuliaServiceGuard, repo_root, reserve_service_port, wait_for_service_ready_with_attempts,
    wendaosearch_julia_project, wendaosearch_script,
};
use crate::{
    GraphStructuralFilterRequestRow, GraphStructuralKeywordOverlapCandidateMetadataInput,
    GraphStructuralKeywordOverlapQueryInput, GraphStructuralKeywordOverlapRawCandidateInput,
    build_graph_structural_filter_request_batch,
    build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs,
    build_graph_structural_keyword_overlap_query_inputs,
    build_graph_structural_keyword_overlap_raw_candidate_inputs,
    fetch_graph_structural_filter_rows_for_repository,
    fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates,
    graph_structural_pair_candidate_id,
};

#[path = "wendaosearch_parser_summary.rs"]
mod parser_summary;
#[path = "wendaosearch_warm_path_stats.rs"]
mod stats;
#[path = "wendaosearch_service_types.rs"]
mod types;

pub use parser_summary::{
    probe_wendaosearch_modelica_parser_summary_route_for_tests,
    spawn_wendaosearch_all_parser_summary_service, spawn_wendaosearch_julia_parser_summary_service,
    spawn_wendaosearch_julia_parser_summary_service_with_attempts,
    spawn_wendaosearch_modelica_parser_summary_service,
};
pub(crate) use stats::warm_path_passes_limits;
use stats::warm_path_stats_from_samples;
pub use types::{
    WendaoSearchGraphStructuralPrewarmReport, WendaoSearchGraphStructuralStabilizationLimits,
    WendaoSearchGraphStructuralStabilizationReason, WendaoSearchGraphStructuralStabilizationReport,
    WendaoSearchGraphStructuralWarmPathStats,
};

const WENDAOSEARCH_SOLVER_DEMO_BASE_URL_ENV: &str = "WENDAOSEARCH_SOLVER_DEMO_BASE_URL";
const WENDAOSEARCH_SOLVER_DEMO_READY_TIMEOUT_SECS: u64 = 90;
const WENDAOSEARCH_SOLVER_DEMO_READY_ATTEMPTS_ENV: &str = "WENDAOSEARCH_SOLVER_DEMO_READY_ATTEMPTS";
const WENDAOSEARCH_SOLVER_DEMO_READY_ATTEMPTS_DEFAULT: usize = 180;

static EXTERNAL_SOLVER_DEMO_READY_URLS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Spawns the managed `WendaoSearch` structural-rerank service in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_structural_rerank_service() -> (String, JuliaServiceGuard) {
    spawn_wendaosearch_service("structural_rerank", "demo").await
}

/// Spawns the managed `WendaoSearch` structural-rerank service in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_structural_rerank_service()
-> (String, JuliaServiceGuard) {
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaServiceGuard::external());
    }
    spawn_wendaosearch_service("structural_rerank", "solver_demo").await
}

/// Spawns the managed same-port multi-route `WendaoSearch` service in `demo`
/// mode.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_demo_multi_route_service() -> (String, JuliaServiceGuard) {
    spawn_wendaosearch_multi_route_service("demo").await
}

/// Spawns the managed same-port multi-route `WendaoSearch` service in
/// `solver_demo` mode.
///
/// # Panics
///
/// Panics when the service script cannot be resolved or the service fails to
/// start.
pub async fn spawn_wendaosearch_solver_demo_multi_route_service() -> (String, JuliaServiceGuard) {
    if let Some(base_url) = configured_solver_demo_base_url() {
        wait_for_external_solver_demo_service(base_url.as_str()).await;
        return (base_url, JuliaServiceGuard::external());
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
    let repositories = graph_structural_stabilization_repositories(base_url);

    let sequential_samples = sample_sequential_graph_structural_routes(
        &repositories,
        &candidate_id,
        limits.sample_count,
    )
    .await?;
    let concurrent_samples = sample_concurrent_graph_structural_routes(
        &repositories,
        &candidate_id,
        limits.sample_count,
    )
    .await?;

    let sequential = warm_path_stats_from_samples(&sequential_samples);
    let concurrent = warm_path_stats_from_samples(&concurrent_samples);
    let stability_reason = graph_structural_stability_reason(&sequential, &concurrent, &limits);
    let stable = stability_reason == WendaoSearchGraphStructuralStabilizationReason::Stable;
    let recommended_max_in_flight = recommended_graph_structural_max_in_flight(stable, &limits);

    Ok(WendaoSearchGraphStructuralStabilizationReport {
        prewarm,
        sequential,
        concurrent,
        stable,
        stability_reason,
        recommended_max_in_flight,
    })
}

struct GraphStructuralStabilizationRepositories {
    explicit_rerank: RegisteredRepository,
    explicit_filter: RegisteredRepository,
    manifest: RegisteredRepository,
}

fn graph_structural_stabilization_repositories(
    base_url: &str,
) -> GraphStructuralStabilizationRepositories {
    GraphStructuralStabilizationRepositories {
        explicit_rerank: graph_structural_explicit_rerank_repository(base_url),
        explicit_filter: graph_structural_explicit_filter_repository(base_url),
        manifest: graph_structural_manifest_repository(base_url),
    }
}

async fn sample_sequential_graph_structural_routes(
    repositories: &GraphStructuralStabilizationRepositories,
    candidate_id: &str,
    sample_count: usize,
) -> Result<Vec<std::time::Duration>, String> {
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let started_at = Instant::now();
        sample_graph_structural_routes_once(repositories, candidate_id, "sequential").await?;
        samples.push(started_at.elapsed());
    }
    Ok(samples)
}

async fn sample_graph_structural_routes_once(
    repositories: &GraphStructuralStabilizationRepositories,
    candidate_id: &str,
    label_prefix: &str,
) -> Result<(), String> {
    prewarm_solver_demo_rerank_route(
        &repositories.explicit_rerank,
        candidate_id,
        &format!("{label_prefix} structural-rerank"),
    )
    .await?;
    prewarm_solver_demo_filter_route(
        &repositories.explicit_filter,
        candidate_id,
        &format!("{label_prefix} constraint-filter"),
    )
    .await?;
    prewarm_solver_demo_rerank_route(
        &repositories.manifest,
        candidate_id,
        &format!("{label_prefix} manifest structural-rerank"),
    )
    .await?;
    prewarm_solver_demo_filter_route(
        &repositories.manifest,
        candidate_id,
        &format!("{label_prefix} manifest constraint-filter"),
    )
    .await
}

async fn sample_concurrent_graph_structural_routes(
    repositories: &GraphStructuralStabilizationRepositories,
    candidate_id: &str,
    sample_count: usize,
) -> Result<Vec<std::time::Duration>, String> {
    let mut samples = Vec::with_capacity(sample_count);
    for _ in 0..sample_count {
        let started_at = Instant::now();
        tokio::try_join!(
            prewarm_solver_demo_rerank_route(
                &repositories.explicit_rerank,
                candidate_id,
                "concurrent structural-rerank",
            ),
            prewarm_solver_demo_filter_route(
                &repositories.explicit_filter,
                candidate_id,
                "concurrent constraint-filter",
            ),
            prewarm_solver_demo_rerank_route(
                &repositories.manifest,
                candidate_id,
                "concurrent manifest structural-rerank",
            ),
            prewarm_solver_demo_filter_route(
                &repositories.manifest,
                candidate_id,
                "concurrent manifest constraint-filter",
            ),
        )?;
        samples.push(started_at.elapsed());
    }
    Ok(samples)
}

fn graph_structural_stability_reason(
    sequential: &WendaoSearchGraphStructuralWarmPathStats,
    concurrent: &WendaoSearchGraphStructuralWarmPathStats,
    limits: &WendaoSearchGraphStructuralStabilizationLimits,
) -> WendaoSearchGraphStructuralStabilizationReason {
    match (
        warm_path_passes_limits(sequential, limits),
        warm_path_passes_limits(concurrent, limits),
    ) {
        (true, true) => WendaoSearchGraphStructuralStabilizationReason::Stable,
        (false, true) => WendaoSearchGraphStructuralStabilizationReason::SequentialExceeded,
        (true, false) => WendaoSearchGraphStructuralStabilizationReason::ConcurrentExceeded,
        (false, false) => WendaoSearchGraphStructuralStabilizationReason::BothExceeded,
    }
}

fn recommended_graph_structural_max_in_flight(
    stable: bool,
    limits: &WendaoSearchGraphStructuralStabilizationLimits,
) -> usize {
    if stable {
        return limits.preferred_max_in_flight.max(1);
    }
    limits
        .degraded_max_in_flight
        .max(1)
        .min(limits.preferred_max_in_flight.max(1))
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
    let ready_urls = EXTERNAL_SOLVER_DEMO_READY_URLS.get_or_init(|| Mutex::new(HashSet::new()));
    let already_ready = {
        ready_urls
            .lock()
            .unwrap_or_else(|error| {
                panic!("solver-demo readiness cache lock failure: {error}");
            })
            .contains(base_url)
    };
    if already_ready {
        return;
    }

    wait_for_service_ready_with_attempts(base_url, wendaosearch_solver_demo_ready_attempts())
        .await
        .unwrap_or_else(|error| {
            panic!(
                "wait for externally managed WendaoSearch solver-demo service readiness: {error}"
            )
        });

    ready_urls
        .lock()
        .unwrap_or_else(|error| {
            panic!("solver-demo readiness cache lock failure: {error}");
        })
        .insert(base_url.to_string());
}

fn wendaosearch_solver_demo_ready_attempts() -> usize {
    env::var(WENDAOSEARCH_SOLVER_DEMO_READY_ATTEMPTS_ENV)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|attempts| *attempts > 0)
        .unwrap_or(WENDAOSEARCH_SOLVER_DEMO_READY_ATTEMPTS_DEFAULT)
}

#[cfg(test)]
#[path = "../../tests/unit/integration_support/wendaosearch_services.rs"]
mod tests;

async fn prewarm_solver_demo_rerank_route(
    repository: &RegisteredRepository,
    candidate_id: &str,
    route_label: &str,
) -> Result<(), String> {
    let rows =
        fetch_graph_structural_keyword_overlap_pair_rerank_rows_for_repository_from_raw_candidates(
            repository,
            &build_graph_structural_keyword_overlap_query_inputs(
                GraphStructuralKeywordOverlapQueryInput {
                    query_id: "query-live".to_string(),
                    retrieval_layer: 0,
                    query_max_layers: 2,
                    keyword_anchors: vec!["alpha".to_string()],
                    edge_constraint_kinds: vec!["depends_on".to_string()],
                },
            ),
            &[build_graph_structural_keyword_overlap_raw_candidate_inputs(
                GraphStructuralKeywordOverlapRawCandidateInput {
                    metadata_inputs:
                        build_graph_structural_keyword_overlap_pair_candidate_metadata_inputs(
                            GraphStructuralKeywordOverlapCandidateMetadataInput {
                                left_id: "node-1".to_string(),
                                right_id: "node-2".to_string(),
                                edge_kinds: vec!["depends_on".to_string()],
                                left_tags: vec!["alpha".to_string(), "core".to_string()],
                                right_tags: vec!["core".to_string()],
                            },
                        ),
                    semantic_score: 0.7,
                    dependency_score: 0.6,
                    keyword_match: true,
                },
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
            id: "julia-code-parser".to_string(),
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
            id: "julia-code-parser".to_string(),
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
            id: "julia-code-parser".to_string(),
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

async fn spawn_wendaosearch_service(route_name: &str, mode: &str) -> (String, JuliaServiceGuard) {
    spawn_wendaosearch_service_with_code_parser_routes(route_name, mode, &[], 120).await
}

async fn spawn_wendaosearch_service_with_code_parser_routes(
    route_name: &str,
    mode: &str,
    code_parser_route_names: &[&str],
    ready_attempts: usize,
) -> (String, JuliaServiceGuard) {
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
    let mut guard = JuliaServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), ready_attempts)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch service readiness: {error}");
        });

    (base_url, guard)
}

async fn spawn_wendaosearch_multi_route_service(mode: &str) -> (String, JuliaServiceGuard) {
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
    let mut guard = JuliaServiceGuard::new(child);

    wait_for_service_ready_with_attempts(base_url.as_str(), 120)
        .await
        .unwrap_or_else(|error| {
            guard.kill();
            panic!("wait for WendaoSearch multi-route service readiness: {error}");
        });

    (base_url, guard)
}
