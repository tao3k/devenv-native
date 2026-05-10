//! Constants for `SearchStrategyFlow` Flight materialization.

pub(super) const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
pub(super) const REPO_SEARCH_LIMIT: usize = 10;
pub(super) const MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS: usize = 32;
pub(super) const MAX_FLIGHT_DISCOVERY_CANDIDATES: usize = 12;
pub(super) const RELATED_CONTEXT_LIMIT: usize = 5;
pub(super) const GRAPH_HOPS: usize = 2;
pub(super) const GRAPH_LIMIT: usize = 50;
