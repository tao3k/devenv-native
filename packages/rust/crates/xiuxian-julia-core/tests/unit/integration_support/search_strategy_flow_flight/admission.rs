use xiuxian_polyglot_orchestrator::JuliaScheduleAction;

use super::{route_materialization_wave_size, search_strategy_flow_materialization_schedule_plan};
use crate::integration_support::SearchStrategyFlowFlightMaterializationConfig;

#[test]
fn search_strategy_flow_flight_materialization_uses_orchestrator_admission() {
    let config = SearchStrategyFlowFlightMaterializationConfig::new_with_backend_default_repo(
        "http://127.0.0.1:12345",
    )
    .unwrap_or_else(|error| panic!("create SearchStrategyFlow Flight config: {error}"));
    let plan = search_strategy_flow_materialization_schedule_plan(32, &config);
    let wave_size = route_materialization_wave_size(32, &config)
        .unwrap_or_else(|error| panic!("admit SearchStrategyFlow materialization: {error}"));

    assert!(
        matches!(
            plan.action,
            JuliaScheduleAction::Dispatch | JuliaScheduleAction::Queue
        ),
        "route materialization must be dispatched or throttled by orchestrator admission"
    );
    assert!(
        wave_size > 0,
        "route materialization must receive a non-zero admitted wave size"
    );
    assert!(
        wave_size <= 32,
        "route materialization wave size must not exceed the route count"
    );
    assert_eq!(
        wave_size,
        usize::try_from(plan.max_in_flight_recommendation)
            .unwrap_or(usize::MAX)
            .max(1)
            .min(32),
        "route materialization must use the orchestrator max-in-flight recommendation"
    );
}

#[test]
fn search_strategy_flow_flight_materialization_allows_empty_trace_without_fanout() {
    let config = SearchStrategyFlowFlightMaterializationConfig::new_with_backend_default_repo(
        "http://127.0.0.1:12345",
    )
    .unwrap_or_else(|error| panic!("create SearchStrategyFlow Flight config: {error}"));

    assert_eq!(
        route_materialization_wave_size(0, &config)
            .unwrap_or_else(|error| panic!("admit empty SearchStrategyFlow trace: {error}")),
        0
    );
}
