use std::env;

use super::{
    RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_PERSISTENT_HOST_TEST_ENV,
    RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV,
    RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV,
    SearchStrategyFlowFlightMaterializationConfig, SearchStrategyFlowPersistentBatchHost,
    SearchStrategyFlowPersistentHostStabilizationLimits,
    WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV,
    WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV,
    WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_INTENT,
    WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT_ENV,
    assert_configured_markdown_live_replay_reports, assert_live_flight_trace_contract,
    live_flight_expected_source_fragments, live_flight_timeout_seconds, optional_non_blank_env,
    required_non_blank_env, run_configured_markdown_live_replay_reports,
    run_wendaograph_search_strategy_flow_json_with_flight_materialization,
    search_strategy_flow_live_replay_search_root,
};

#[test]
fn wendaograph_search_strategy_flow_live_replay_runs_local_markdown_families_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow live replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_REPLAY_TEST_ENV}=1"
        );
        return;
    }

    let search_root = search_strategy_flow_live_replay_search_root();
    let report = run_configured_markdown_live_replay_reports(search_root.as_path(), None);
    assert_configured_markdown_live_replay_reports(&report.family_reports);
}

#[tokio::test]
async fn wendaograph_search_strategy_flow_live_flight_index_replay_runs_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV).is_none() {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow live Flight replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_REPLAY_TEST_ENV}=1, {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV}, and {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV}"
        );
        return;
    }

    let base_url = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV);
    let repo_id = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV);
    let timeout_seconds = live_flight_timeout_seconds();
    let intent = optional_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT_ENV)
        .unwrap_or_else(|| WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_INTENT.to_owned());
    let expected_source_fragments = live_flight_expected_source_fragments();
    let search_root = search_strategy_flow_live_replay_search_root();
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, repo_id)
        .unwrap_or_else(|error| panic!("create live Flight materialization config: {error}"))
        .with_timeout_seconds(timeout_seconds);

    let trace = run_wendaograph_search_strategy_flow_json_with_flight_materialization(
        &intent,
        search_root.as_path(),
        Some(config),
    )
    .await
    .unwrap_or_else(|error| panic!("run live SearchStrategyFlow Flight index replay: {error}"));
    let trace = serde_json::from_str::<serde_json::Value>(&trace)
        .unwrap_or_else(|error| panic!("parse live Flight replay trace: {error}"));
    assert_live_flight_trace_contract("live-flight-index", &trace, &expected_source_fragments);
}

#[tokio::test]
async fn wendaograph_search_strategy_flow_live_flight_persistent_host_runs_when_enabled() {
    if env::var_os(RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_PERSISTENT_HOST_TEST_ENV)
        .is_none()
    {
        eprintln!(
            "skipping WendaoGraph SearchStrategyFlow persistent live Flight replay; set {RUN_WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_PERSISTENT_HOST_TEST_ENV}=1, {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV}, and {WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV}"
        );
        return;
    }

    let base_url = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_BASE_URL_ENV);
    let repo_id = required_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_FLIGHT_REPO_ID_ENV);
    let timeout_seconds = live_flight_timeout_seconds();
    let intent = optional_non_blank_env(WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_INTENT_ENV)
        .unwrap_or_else(|| WENDAOGRAPH_SEARCH_STRATEGY_FLOW_LIVE_FLIGHT_DEFAULT_INTENT.to_owned());
    let search_root = search_strategy_flow_live_replay_search_root();
    let config = SearchStrategyFlowFlightMaterializationConfig::new(base_url, repo_id)
        .unwrap_or_else(|error| {
            panic!("create persistent live Flight materialization config: {error}")
        })
        .with_timeout_seconds(timeout_seconds);
    let mut host = SearchStrategyFlowPersistentBatchHost::start(search_root.as_path())
        .unwrap_or_else(|error| panic!("start persistent SearchStrategyFlow host: {error}"));

    let report = host
        .stabilize_with_flight_materialization(
            &intent,
            &config,
            SearchStrategyFlowPersistentHostStabilizationLimits::default().with_sample_count(1),
        )
        .await
        .unwrap_or_else(|error| panic!("stabilize persistent live Flight host: {error}"));
    host.finish()
        .unwrap_or_else(|error| panic!("finish persistent SearchStrategyFlow host: {error}"));

    assert_eq!(report.warm.sample_count, 1);
    assert!(report.recommended_max_in_flight >= 1);
    eprintln!(
        "SearchStrategyFlow persistent live Flight release summary: prewarmMs={}, warmP95Ms={:.3}, warmMaxMs={:.3}, stable={}, reason={:?}, recommendedMaxInFlight={}",
        report.prewarm_elapsed.as_millis(),
        report.warm.p95_ms,
        report.warm.max_ms,
        report.stable,
        report.stability_reason,
        report.recommended_max_in_flight
    );
}
