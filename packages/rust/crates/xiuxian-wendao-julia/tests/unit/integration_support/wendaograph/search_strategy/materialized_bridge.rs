use std::{env, path::PathBuf, time::Instant};

use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowMaterializedRepoReplayFamily,
    materialized_search_strategy_flow_markdown_replay_families_from_bridge_report,
};

use super::{
    SearchStrategyFlowPersistentBatchHost,
    assert_search_strategy_flow_live_replay_trace_with_candidate_source,
    run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches,
    search_strategy_flow_live_replay_search_root,
};

const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_BATCH_REPLAY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_BATCH_REPLAY_TEST";
const RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_PERSISTENT_BATCH_REPLAY_TEST_ENV: &str =
    "RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_PERSISTENT_BATCH_REPLAY_TEST";

#[test]
fn wendaograph_search_strategy_flow_materialized_batch_replay_runs_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_BATCH_REPLAY_TEST_ENV)
        .is_none()
    {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow materialized batch replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_BATCH_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report_path = materialized_bridge_audit_report_path();
    let intent = materialized_replay_intent();
    let max_repos = materialized_replay_max_repos(16);
    let max_candidates_per_repo = materialized_replay_max_candidates_per_repo();
    let families = materialized_search_strategy_flow_markdown_replay_families_from_bridge_report(
        &report_path,
        &intent,
        Some(max_repos),
        Some(max_candidates_per_repo),
    )
    .unwrap_or_else(|error| {
        panic!("build materialized SearchStrategyFlow replay families: {error}")
    });
    assert!(
        !families.is_empty(),
        "materialized batch replay requires benchmark-ready repo families"
    );

    let candidate_batches = families
        .iter()
        .map(|family| (intent.as_str(), family.batch.clone()))
        .collect::<Vec<_>>();
    let started = Instant::now();
    let traces = run_wendaograph_search_strategy_flow_json_batch_with_candidate_batches(
        search_root,
        candidate_batches,
    )
    .unwrap_or_else(|error| panic!("run materialized SearchStrategyFlow batch replay: {error}"));
    let elapsed_ms = started.elapsed().as_millis();
    let report = assert_materialized_batch_traces(&families, &traces);

    eprintln!(
        "SearchStrategyFlow materialized batch replay summary: repos={}, inputCandidates={}, selectedFrontier={}, routes={}, projectedRows={}, elapsedMs={}",
        families.len(),
        report.input_candidate_count,
        report.selected_frontier_count,
        report.route_count,
        report.projected_row_count,
        elapsed_ms
    );
}

#[test]
fn wendaograph_search_strategy_flow_materialized_persistent_batch_replay_runs_when_enabled() {
    if env::var_os(
        RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_PERSISTENT_BATCH_REPLAY_TEST_ENV,
    )
    .is_none()
    {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow materialized persistent batch replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_MATERIALIZED_PERSISTENT_BATCH_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report_path = materialized_bridge_audit_report_path();
    let intent = materialized_replay_intent();
    let max_repos = materialized_replay_max_repos(32);
    let max_candidates_per_repo = materialized_replay_max_candidates_per_repo();
    let families = materialized_search_strategy_flow_markdown_replay_families_from_bridge_report(
        &report_path,
        &intent,
        Some(max_repos),
        Some(max_candidates_per_repo),
    )
    .unwrap_or_else(|error| {
        panic!("build materialized SearchStrategyFlow replay families: {error}")
    });
    assert!(
        !families.is_empty(),
        "materialized persistent batch replay requires benchmark-ready repo families"
    );

    let candidate_batches = || {
        families
            .iter()
            .map(|family| (intent.as_str(), family.batch.clone()))
            .collect::<Vec<_>>()
    };
    let mut host =
        SearchStrategyFlowPersistentBatchHost::start(search_root).unwrap_or_else(|error| {
            panic!("start persistent materialized SearchStrategyFlow host: {error}")
        });

    let cold_started = Instant::now();
    let cold_traces = host.submit(candidate_batches()).unwrap_or_else(|error| {
        panic!("run cold persistent materialized SearchStrategyFlow replay: {error}")
    });
    let cold_elapsed_ms = cold_started.elapsed().as_millis();
    let cold_report = assert_materialized_batch_traces(&families, &cold_traces);

    let warm_started = Instant::now();
    let warm_traces = host.submit(candidate_batches()).unwrap_or_else(|error| {
        panic!("run warm persistent materialized SearchStrategyFlow replay: {error}")
    });
    let warm_elapsed_ms = warm_started.elapsed().as_millis();
    let warm_report = assert_materialized_batch_traces(&families, &warm_traces);

    host.finish().unwrap_or_else(|error| {
        panic!("finish persistent materialized SearchStrategyFlow host: {error}")
    });
    assert_eq!(
        cold_report.input_candidate_count,
        warm_report.input_candidate_count
    );
    assert_eq!(cold_report.route_count, warm_report.route_count);
    assert!(
        warm_elapsed_ms < cold_elapsed_ms,
        "warm persistent submit should be faster than cold submit, cold={cold_elapsed_ms}ms warm={warm_elapsed_ms}ms"
    );
    eprintln!(
        "SearchStrategyFlow materialized persistent batch replay summary: repos={}, inputCandidates={}, selectedFrontier={}, routes={}, projectedRows={}, coldMs={}, warmMs={}",
        families.len(),
        warm_report.input_candidate_count,
        warm_report.selected_frontier_count,
        warm_report.route_count,
        warm_report.projected_row_count,
        cold_elapsed_ms,
        warm_elapsed_ms
    );
}

fn materialized_bridge_audit_report_path() -> PathBuf {
    env::var_os("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_BRIDGE_AUDIT_REPORT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env::var_os("PRJ_CACHE_HOME").unwrap_or_else(|| ".cache".into())).join(
                "agent/reports/2026-05-10-wendaograph-real-scenario-rust-bridge-resource-audit.json",
            )
        })
}

#[derive(Debug, Clone, Copy)]
struct MaterializedBatchReport {
    input_candidate_count: usize,
    selected_frontier_count: usize,
    route_count: usize,
    projected_row_count: usize,
}

fn materialized_replay_intent() -> String {
    env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_INTENT")
        .unwrap_or_else(|_| "SearchStrategyFlow PageIndex LinkGraph evidence strategy".to_owned())
}

fn materialized_replay_max_repos(default: usize) -> usize {
    env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_MAX_REPOS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn materialized_replay_max_candidates_per_repo() -> usize {
    env::var("WENDAOGRAPH_SEARCH_STRATEGY_FLOW_REPLAY_MAX_CANDIDATES_PER_REPO")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(8)
}

fn assert_materialized_batch_traces(
    families: &[SearchStrategyFlowMaterializedRepoReplayFamily],
    traces: &[String],
) -> MaterializedBatchReport {
    assert_eq!(
        traces.len(),
        families.len(),
        "materialized batch replay must return one trace per repo family"
    );

    let mut input_candidate_count = 0;
    let mut selected_frontier_count = 0;
    let mut route_count = 0;
    let mut projected_row_count = 0;
    for (family, trace) in families.iter().zip(traces.iter()) {
        let expected_prefix = format!("repos/{}/", family.repo_id);
        let report = assert_search_strategy_flow_live_replay_trace_with_candidate_source(
            "materialized-repo",
            "rust-materialized-repo-markdown-headings",
            false,
            false,
            &expected_prefix,
            family.markdown_file_count,
            family.heading_count,
            family.batch.row_count,
            trace,
        );
        input_candidate_count += report.input_candidate_count;
        selected_frontier_count += report.selected_frontier_count;
        route_count += report.route_count;
        projected_row_count += report.projected_row_count;
    }

    MaterializedBatchReport {
        input_candidate_count,
        selected_frontier_count,
        route_count,
        projected_row_count,
    }
}
