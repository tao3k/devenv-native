use xiuxian_polyglot_orchestrator::JuliaScheduleAction;
use xiuxian_wendao_core::transport::PluginTransportKind;

use super::fixtures::fixture_candidate_batch;
use crate::integration_support::{
    SearchStrategyFlowServiceFlightBindingOptions,
    build_search_strategy_flow_service_flight_binding,
    build_search_strategy_flow_service_orchestrator_schedule_plan,
};

#[test]
fn search_strategy_flow_service_binding_uses_arrow_route_and_orchestrator_admission() {
    let candidate_batch = fixture_candidate_batch();
    let options = binding_options("http://127.0.0.1:8815").with_max_in_flight_requests(4);

    let binding = must(
        build_search_strategy_flow_service_flight_binding(options, &candidate_batch),
        "service binding should build",
    );

    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some("/wendao/graph/search_strategy_flow")
    );
    assert_eq!(
        binding.contract_version.0,
        "xiuxian_wendao.graph.search_strategy_flow.service.v1"
    );
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(4));
    assert_eq!(binding.transport, PluginTransportKind::ArrowFlight);
    assert!(
        !candidate_batch.candidate_input_arrow_ipc_stream.is_empty(),
        "candidate batch should carry Arrow IPC"
    );
}

#[test]
fn search_strategy_flow_service_binding_rejects_empty_candidate_batch() {
    let mut candidate_batch = fixture_candidate_batch();
    candidate_batch.row_count = 0;
    let options = binding_options("http://127.0.0.1:8815");

    let error = must_err(build_search_strategy_flow_service_flight_binding(
        options,
        &candidate_batch,
    ));

    assert!(
        error.contains("must include candidate rows"),
        "unexpected error: {error}"
    );
}

#[test]
fn search_strategy_flow_service_schedule_allows_candidate_batch() {
    let candidate_batch = fixture_candidate_batch();
    let options = binding_options("http://127.0.0.1:8815");

    let plan = build_search_strategy_flow_service_orchestrator_schedule_plan(
        &options,
        candidate_batch.row_count,
        candidate_batch.candidate_input_arrow_ipc_byte_len(),
    );

    assert!(matches!(
        plan.action,
        JuliaScheduleAction::Dispatch | JuliaScheduleAction::Queue
    ));
    assert_eq!(plan.profile_id.as_str(), "wendaograph.search_strategy_flow");
}

fn binding_options(base_url: &str) -> SearchStrategyFlowServiceFlightBindingOptions {
    must(
        SearchStrategyFlowServiceFlightBindingOptions::new(base_url),
        "base URL should parse",
    )
}

fn must<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_err<T: std::fmt::Debug, E>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("expected error, got {value:?}"),
        Err(error) => error,
    }
}
