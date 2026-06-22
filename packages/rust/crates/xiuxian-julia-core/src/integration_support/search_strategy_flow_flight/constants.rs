//! Constants for `SearchStrategyFlow` Flight materialization.

pub(super) const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SCHEMA_VERSION: &str =
    "xiuxian_wendao.graph.search_strategy_flow.service.v1";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_SERVICE: &str =
    "wendao.graph.v1.SearchStrategyFlow";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_METHOD: &str = "RunStrategyFlow";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ARROW_IPC_MIME: &str =
    "application/vnd.apache.arrow.stream";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_REQUEST_BUNDLE_TABLE: &str =
    "search_strategy_flow_request";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE: &str =
    "search_strategy_flow_response";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN: &str =
    "strategy_candidates_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN: &str =
    "strategy_transitions_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN: &str =
    "strategy_frontier_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN: &str =
    "strategy_planner_actions_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_QUERY_UNDERSTANDING_PAYLOAD_COLUMN: &str =
    "query_understanding_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ONTOLOGY_REGISTRY_PAYLOAD_COLUMN: &str =
    "ontology_registry_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_BRANCH_JUDGEMENTS_PAYLOAD_COLUMN: &str =
    "branch_judgements_payload";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_ROUTE: &str =
    "/wendao/graph/search_strategy_flow";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROVIDER_ID: &str = "wendaograph";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_CAPABILITY_ID: &str = "search-strategy-flow";
pub(crate) const WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_PROFILE_ID: &str =
    "wendaograph.search_strategy_flow";
pub(super) const REPO_SEARCH_LIMIT: usize = 24;
pub(super) const MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS: usize = 32;
pub(super) const MIN_FLIGHT_REQUIRED_EVIDENCE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP: usize = 12;
pub(super) const MIN_FLIGHT_REQUIRED_EVIDENCE_CANDIDATES_BEFORE_EARLY_STOP: usize = 16;
pub(super) const MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP: usize = 24;
pub(super) const MAX_FLIGHT_DISCOVERY_CANDIDATES: usize = 32;
pub(super) const MAX_FLIGHT_REQUIRED_EVIDENCE_FRONTIER_CANDIDATES: usize = 6;
pub(super) const RELATED_CONTEXT_LIMIT: usize = 5;
pub(super) const GRAPH_HOPS: usize = 1;
pub(super) const GRAPH_LIMIT: usize = 12;

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_flight/constants.rs"]
mod tests;
