//! Arrow Flight client helpers for Studio-backed `SearchStrategyFlow` routes.

mod candidate_source;
mod client;
mod config;
mod constants;
mod ids;
mod materialization;
mod metadata;
mod query;
mod rows;

pub(crate) use candidate_source::search_strategy_flow_candidate_input_batch_from_repo_search;
pub use config::SearchStrategyFlowFlightMaterializationConfig;
pub use materialization::materialize_search_strategy_flow_routes;
