//! Runtime Flight service bridge for `WendaoGraph` `SearchStrategyFlow`.

mod decode;
mod runtime;
mod types;

pub use decode::decode_search_strategy_flow_frontier_rows;
pub use runtime::{
    SearchStrategyFlowServiceFlightBindingOptions,
    build_search_strategy_flow_service_flight_binding,
    build_search_strategy_flow_service_orchestrator_schedule_plan,
    roundtrip_search_strategy_flow_frontier_with_service,
    roundtrip_search_strategy_flow_frontier_with_service_request,
    wendaograph_search_strategy_flow_provider_selector,
    wendaograph_search_strategy_flow_route_profile_ref,
};
pub use types::{
    SearchStrategyFlowFrontierRow, SearchStrategyFlowServiceCandidateRow,
    SearchStrategyFlowServicePlannerActionRow, SearchStrategyFlowServiceResponse,
    SearchStrategyFlowServiceRoundtrip,
};

#[cfg(test)]
#[path = "../../../../tests/unit/integration_support/search_strategy_flow_flight/service.rs"]
mod tests;
