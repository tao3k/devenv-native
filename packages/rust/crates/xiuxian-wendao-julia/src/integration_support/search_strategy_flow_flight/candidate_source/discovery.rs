use std::time::Instant;

use xiuxian_wendao_runtime::transport::REPO_SEARCH_ROUTE;

use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInputBatch, WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};

use crate::integration_support::search_strategy_flow_flight::client::SearchStrategyFlowFlightClient;
use crate::integration_support::search_strategy_flow_flight::config::SearchStrategyFlowFlightMaterializationConfig;
use crate::integration_support::search_strategy_flow_flight::constants::{
    MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS, MAX_FLIGHT_DISCOVERY_CANDIDATES, REPO_SEARCH_LIMIT,
};
use crate::integration_support::search_strategy_flow_flight::metadata::populate_repo_search_headers;
use crate::integration_support::search_strategy_flow_flight::query::candidate_discovery_queries;
use crate::integration_support::search_strategy_flow_flight::rows::{
    repo_relative_candidate_inputs, repo_search_batches_to_candidate_inputs, row_count,
};

use super::evidence::CandidateDiscoveryRequiredEvidence;
use super::exact_seed::{
    apply_exact_markdown_attempt_score_floor, candidate_from_exact_markdown_attempt,
};
use super::frontier::{
    retain_required_evidence_frontier, retain_unique_candidate_sources,
    should_stop_candidate_discovery,
};
use super::merge::merge_candidate_discovery_result;
use super::ranking::{
    calibrate_candidate_discovery_scores_for_intent, rank_candidate_discovery_results_for_intent,
};
use super::receipt::{
    candidate_discovery_attempt_receipt, candidate_discovery_receipt, elapsed_ms,
};

pub(crate) async fn search_strategy_flow_candidate_input_batch_from_repo_search(
    intent: &str,
    config: &SearchStrategyFlowFlightMaterializationConfig,
) -> Result<SearchStrategyFlowCandidateInputBatch, String> {
    let discovery_started_at = Instant::now();
    let mut client = SearchStrategyFlowFlightClient::connect(config).await?;
    let attempts = candidate_discovery_queries(intent);
    let required_evidence = CandidateDiscoveryRequiredEvidence::from_intent(intent);
    let mut attempted = Vec::new();
    let mut attempt_receipts = Vec::new();
    let mut merged_candidates = Vec::new();
    for (attempt_index, attempt) in attempts
        .iter()
        .take(MAX_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS)
        .enumerate()
    {
        attempted.push(format!(
            "query=`{}` prefix=`{}`",
            attempt.query, attempt.path_prefix
        ));
        let attempt_started_at = Instant::now();
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
            elapsed_ms(attempt_started_at),
        ));
        let mut attempt_candidates = repo_relative_candidate_inputs(
            config.repo_id.as_str(),
            repo_search_batches_to_candidate_inputs(&batches),
        );
        if attempt_candidates.is_empty()
            && let Some(candidate) = candidate_from_exact_markdown_attempt(attempt)
        {
            attempt_candidates.push(candidate);
        }
        apply_exact_markdown_attempt_score_floor(&mut attempt_candidates, attempt);
        for candidate in attempt_candidates {
            merge_candidate_discovery_result(&mut merged_candidates, candidate);
        }
        if should_stop_candidate_discovery(attempt_index + 1, &merged_candidates, required_evidence)
        {
            break;
        }
    }
    if !merged_candidates.is_empty() {
        calibrate_candidate_discovery_scores_for_intent(&mut merged_candidates, intent);
        rank_candidate_discovery_results_for_intent(&mut merged_candidates, intent);
        retain_unique_candidate_sources(&mut merged_candidates);
        retain_required_evidence_frontier(&mut merged_candidates, required_evidence);
        merged_candidates.truncate(MAX_FLIGHT_DISCOVERY_CANDIDATES);
        return search_strategy_flow_candidate_input_batch_with_discovery_receipt(
            WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE,
            &merged_candidates,
            &candidate_discovery_receipt(
                config.repo_id.as_str(),
                merged_candidates.len(),
                elapsed_ms(discovery_started_at),
                &attempt_receipts,
            ),
        );
    }

    Err(format!(
        "SearchStrategyFlow repo-search candidate discovery returned zero page-index-ready candidate rows after {} attempts: {}",
        attempted.len(),
        attempted.join("; ")
    ))
}
