//! Constants for `SearchStrategyFlow` Flight materialization.

pub(super) const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
pub(super) const REPO_SEARCH_LIMIT: usize = 10;
pub(super) const MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS: usize = 32;
pub(super) const MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP: usize = 20;
pub(super) const MAX_FLIGHT_DISCOVERY_CANDIDATES: usize = 12;
pub(super) const RELATED_CONTEXT_LIMIT: usize = 5;
pub(super) const GRAPH_HOPS: usize = 1;
pub(super) const GRAPH_LIMIT: usize = 12;

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/constants.rs"]
mod tests;
