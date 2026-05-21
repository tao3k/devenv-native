use std::time::Duration;

use super::{
    SearchStrategyFlowPersistentHostStabilizationLimits,
    SearchStrategyFlowPersistentHostStabilizationReason,
    SearchStrategyFlowPersistentHostStabilizationReport, run_wendaograph_search_strategy_flow_json,
    search_strategy_flow_persistent_host_stats,
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
fn search_strategy_flow_persistent_host_stability_limits_recommend_admission_budget() {
    let limits = SearchStrategyFlowPersistentHostStabilizationLimits {
        max_p95_ms: 200.0,
        max_max_ms: 400.0,
        max_spread_ratio: 4.0,
        preferred_max_in_flight: 2,
        degraded_max_in_flight: 1,
        ..SearchStrategyFlowPersistentHostStabilizationLimits::default()
    };
    let stable = search_strategy_flow_persistent_host_stats(40.0, 80.0, 120.0, 3.0);
    let high_p95 = search_strategy_flow_persistent_host_stats(40.0, 201.0, 220.0, 3.0);
    let high_max = search_strategy_flow_persistent_host_stats(40.0, 199.0, 401.0, 3.0);
    let high_spread = search_strategy_flow_persistent_host_stats(40.0, 199.0, 220.0, 5.0);

    assert_eq!(
        limits.stability_reason_for(&stable),
        SearchStrategyFlowPersistentHostStabilizationReason::Stable
    );
    assert_eq!(
        limits.stability_reason_for(&high_p95),
        SearchStrategyFlowPersistentHostStabilizationReason::P95Exceeded
    );
    assert_eq!(
        limits.stability_reason_for(&high_max),
        SearchStrategyFlowPersistentHostStabilizationReason::MaxExceeded
    );
    assert_eq!(
        limits.stability_reason_for(&high_spread),
        SearchStrategyFlowPersistentHostStabilizationReason::SpreadExceeded
    );
}

#[test]
fn search_strategy_flow_persistent_host_stability_report_exports_json_evidence() {
    let report = SearchStrategyFlowPersistentHostStabilizationReport {
        prewarm_elapsed: Duration::from_millis(123),
        warm: search_strategy_flow_persistent_host_stats(40.0, 80.0, 120.0, 3.0),
        stable: true,
        stability_reason: SearchStrategyFlowPersistentHostStabilizationReason::Stable,
        recommended_max_in_flight: 2,
    };

    let evidence = report.to_json_value();

    assert_eq!(evidence["prewarmElapsedMs"], 123.0);
    assert_eq!(evidence["warm"]["sampleCount"], 3);
    assert_eq!(evidence["warm"]["p95Ms"], 80.0);
    assert_eq!(evidence["stable"], true);
    assert_eq!(evidence["stabilityReason"], "stable");
    assert_eq!(evidence["recommendedMaxInFlight"], 2);
}
