//! Arrow Flight client helpers for Studio-backed `SearchStrategyFlow` routes.

mod admission;
mod candidate_source;
mod client;
mod config;
mod constants;
mod ids;
mod ipc_file;
mod materialization;
mod metadata;
mod ontology_registry;
mod query;
mod request;
mod rows;
mod service;

pub(crate) use candidate_source::search_strategy_flow_candidate_input_batch_from_repo_search;
pub use config::SearchStrategyFlowFlightMaterializationConfig;
pub(crate) use ipc_file::SearchStrategyFlowArrowIpcFile;
pub use materialization::materialize_search_strategy_flow_routes;
pub(crate) use ontology_registry::search_strategy_flow_ontology_registry_arrow_ipc_from_semantic_scope;
pub use request::{
    SearchStrategyFlowServiceArrowRequest, SearchStrategyFlowServiceRequestOptions,
    build_search_strategy_flow_service_arrow_request,
    build_search_strategy_flow_service_flight_request_batch,
};
pub use service::{
    SearchStrategyFlowFrontierRow, SearchStrategyFlowServiceCandidateRow,
    SearchStrategyFlowServiceFlightBindingOptions, SearchStrategyFlowServicePlannerActionRow,
    SearchStrategyFlowServiceResponse, SearchStrategyFlowServiceRoundtrip,
    build_search_strategy_flow_service_flight_binding,
    build_search_strategy_flow_service_orchestrator_schedule_plan,
    decode_search_strategy_flow_frontier_rows,
    roundtrip_search_strategy_flow_frontier_with_service,
    roundtrip_search_strategy_flow_frontier_with_service_request,
    wendaograph_search_strategy_flow_provider_selector,
    wendaograph_search_strategy_flow_route_profile_ref,
};
