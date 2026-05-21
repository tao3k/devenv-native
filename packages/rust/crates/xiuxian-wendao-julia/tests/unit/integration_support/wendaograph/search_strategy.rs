include!("search_strategy/support.rs");

#[path = "search_strategy/basic.rs"]
mod basic;
#[path = "search_strategy/batch_profile.rs"]
mod batch_profile;
#[path = "search_strategy/candidate_discovery.rs"]
mod candidate_discovery;
#[path = "search_strategy/flight_materialization.rs"]
mod flight_materialization;
#[path = "search_strategy/live_flight.rs"]
mod live_flight;
#[path = "search_strategy/materialized_bridge.rs"]
mod materialized_bridge;
#[path = "search_strategy/probe_actions.rs"]
mod probe_actions;
#[path = "search_strategy/registry.rs"]
mod registry;
#[path = "search_strategy/required_evidence.rs"]
mod required_evidence;
#[path = "search_strategy/retrieval_routes.rs"]
mod retrieval_routes;
#[path = "search_strategy/service_boundary.rs"]
mod service_boundary;
