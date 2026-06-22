use std::time::Instant;

use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::{REPO_SEARCH_ROUTE, WENDAO_ARROW_FLIGHT_DATA_PLANE};

use crate::integration_support::search_strategy_flow_candidates::WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE;
use crate::integration_support::search_strategy_flow_flight::constants::{
    MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS, MAX_FLIGHT_DISCOVERY_CANDIDATES, REPO_SEARCH_LIMIT,
};

pub(super) fn candidate_discovery_attempt_receipt(
    query: &str,
    path_prefix: &str,
    row_count: usize,
    elapsed_ms: u128,
) -> Value {
    json!({
        "route": REPO_SEARCH_ROUTE,
        "query": query,
        "pathPrefix": path_prefix,
        "requestLimit": REPO_SEARCH_LIMIT,
        "rowCount": row_count,
        "elapsedMs": elapsed_ms,
    })
}

pub(super) fn candidate_discovery_receipt(
    repo_id: &str,
    merged_candidate_count: usize,
    elapsed_ms: u128,
    attempts: &[Value],
) -> Value {
    json!({
        "receiptSource": WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE,
        "candidateInputSource": WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE,
        "candidateInputCount": merged_candidate_count,
        "repoId": repo_id,
        "transport": WENDAO_ARROW_FLIGHT_DATA_PLANE,
        "route": REPO_SEARCH_ROUTE,
        "retrievalOwner": "wendao-gateway",
        "candidateDiscoveryMode": "repo-search-page-index-link-graph-frontier",
        "requestLimit": REPO_SEARCH_LIMIT,
        "attemptCount": attempts.len(),
        "maxAttemptCount": MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS,
        "maxMergedCandidateCount": MAX_FLIGHT_DISCOVERY_CANDIDATES,
        "mergedCandidateCount": merged_candidate_count,
        "elapsedMs": elapsed_ms,
        "attempts": attempts,
    })
}

pub(super) fn elapsed_ms(started_at: Instant) -> u128 {
    started_at.elapsed().as_millis()
}
