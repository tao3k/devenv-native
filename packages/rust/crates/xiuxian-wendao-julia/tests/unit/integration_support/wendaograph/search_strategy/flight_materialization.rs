use super::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    GRAPH_NEIGHBORS_ROUTE, REPO_SEARCH_ROUTE, SearchStrategyFlowFakeFlightScenario,
    SearchStrategyFlowFlightMaterializationConfig,
    enrich_wendaograph_search_strategy_flow_retrieval_routes_with_flight_materialization,
    spawn_fake_search_strategy_flow_flight_service,
    spawn_fake_search_strategy_flow_flight_service_for,
    spawn_fake_search_strategy_flow_flight_service_with_empty_repo_search,
    spawn_fake_search_strategy_flow_flight_service_with_graph_node_allowlist,
    spawn_fake_search_strategy_flow_flight_service_without_page_index,
};

#[path = "flight_materialization_cases/code_substitute.rs"]
mod code_substitute;
#[path = "flight_materialization_cases/core.rs"]
mod core;
#[path = "flight_materialization_cases/missing_projected_page.rs"]
mod missing_projected_page;
#[path = "flight_materialization_cases/reference_routes.rs"]
mod reference_routes;
#[path = "flight_materialization_cases/structured_source.rs"]
mod structured_source;
