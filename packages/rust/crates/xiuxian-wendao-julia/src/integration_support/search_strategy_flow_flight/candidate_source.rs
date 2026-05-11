use std::collections::HashSet;

use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::REPO_SEARCH_ROUTE;

use crate::integration_support::search_strategy_flow_candidates::{
    FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInputBatch,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};

use super::client::SearchStrategyFlowFlightClient;
use super::config::SearchStrategyFlowFlightMaterializationConfig;
use super::constants::{
    MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS, MAX_FLIGHT_DISCOVERY_CANDIDATES, REPO_SEARCH_LIMIT,
};
use super::metadata::populate_repo_search_headers;
use super::query::candidate_discovery_queries;
use super::rows::{
    repo_relative_candidate_inputs, repo_search_batches_to_candidate_inputs, row_count,
};

pub(crate) async fn search_strategy_flow_candidate_input_batch_from_repo_search(
    intent: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let attempts = candidate_discovery_queries(intent);
    let mut attempted = Vec::new();
    let mut attempt_receipts = Vec::new();
    let mut seen = HashSet::<(String, String)>::new();
    let mut merged_candidates = Vec::new();
    for attempt in attempts
        .iter()
        .take(MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS)
    {
        attempted.push(format!(
            "query=`{}` prefix=`{}`",
            attempt.query, attempt.path_prefix
        ));
        let batches = client
            .collect_route_batches_allow_empty(
                REPO_SEARCH_ROUTE,
                "SearchStrategyFlow repo-search candidate discovery",
                |metadata| {
                    populate_repo_search_headers(
                        metadata,
                        &config.repo_id,
                        attempt.query.as_str(),
                        REPO_SEARCH_LIMIT,
                        attempt.path_prefix.as_str(),
                    )
                },
            )
            .await?;
        attempt_receipts.push(candidate_discovery_attempt_receipt(
            attempt.query.as_str(),
            attempt.path_prefix.as_str(),
            row_count(&batches),
        ));
        for candidate in repo_relative_candidate_inputs(
            config.repo_id.as_str(),
            repo_search_batches_to_candidate_inputs(&batches),
        ) {
            let key = (
                candidate.relative_path.clone(),
                candidate.heading_anchor.clone(),
            );
            if seen.insert(key) {
                merged_candidates.push(candidate);
            }
            if merged_candidates.len() >= MAX_FLIGHT_DISCOVERY_CANDIDATES {
                return Ok(
                    search_strategy_flow_candidate_input_batch_with_discovery_receipt(
                        FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
                        &merged_candidates,
                        candidate_discovery_receipt(
                            config.repo_id.as_str(),
                            merged_candidates.len(),
                            attempt_receipts,
                        ),
                    ),
                );
            }
        }
    }
    if !merged_candidates.is_empty() {
        return Ok(
            search_strategy_flow_candidate_input_batch_with_discovery_receipt(
                FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
                &merged_candidates,
                candidate_discovery_receipt(
                    config.repo_id.as_str(),
                    merged_candidates.len(),
                    attempt_receipts,
                ),
            ),
        );
    }

    Err(format!(
        "SearchStrategyFlow repo-search candidate discovery returned zero page-index-ready candidate rows after {} attempts: {}",
        attempted.len(),
        attempted.join("; ")
    ))
}

fn candidate_discovery_attempt_receipt(query: &str, path_prefix: &str, row_count: usize) -> Value {
    json!({
        "route": REPO_SEARCH_ROUTE,
        "query": query,
        "pathPrefix": path_prefix,
        "requestLimit": REPO_SEARCH_LIMIT,
        "rowCount": row_count,
    })
}

fn candidate_discovery_receipt(
    repo_id: &str,
    merged_candidate_count: usize,
    attempts: Vec<Value>,
) -> Value {
    json!({
        "receiptSource": FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
        "candidateInputSource": FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE,
        "candidateInputCount": merged_candidate_count,
        "repoId": repo_id,
        "transport": "arrow-flight",
        "route": REPO_SEARCH_ROUTE,
        "requestLimit": REPO_SEARCH_LIMIT,
        "attemptCount": attempts.len(),
        "maxAttemptCount": MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS,
        "maxMergedCandidateCount": MAX_FLIGHT_DISCOVERY_CANDIDATES,
        "mergedCandidateCount": merged_candidate_count,
        "attempts": attempts,
    })
}
