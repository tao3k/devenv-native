use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInput, WENDAO_GATEWAY_RETRIEVAL_CANDIDATE_SOURCE,
};
pub(super) use crate::integration_support::search_strategy_flow_flight::query::RepoSearchAttempt;

use super::{
    CandidateDiscoveryRankingMode, apply_exact_markdown_attempt_score_floor,
    calibrate_candidate_discovery_scores, candidate_discovery_attempt_receipt,
    candidate_discovery_priority, candidate_discovery_receipt,
    candidate_from_exact_markdown_attempt, candidate_matches_relation_path_evidence,
    merge_candidate_discovery_result, rank_candidate_discovery_results,
    rank_candidate_discovery_results_for_intent, ranking_mode_for_intent,
    retain_unique_candidate_sources, should_stop_candidate_discovery,
};

mod early_stop;
mod merge;
mod ranking;
mod receipt;

fn candidate(
    relative_path: &'static str,
    title: &'static str,
    score: f64,
) -> SearchStrategyFlowCandidateInput {
    candidate_with_edges(relative_path, title, score, &[])
}

fn candidate_with_edges(
    relative_path: &'static str,
    title: &'static str,
    score: f64,
    edge_kinds: &[&str],
) -> SearchStrategyFlowCandidateInput {
    SearchStrategyFlowCandidateInput {
        relative_path: relative_path.to_owned(),
        heading_anchor: title.to_ascii_lowercase().replace(' ', "-"),
        title: title.to_owned(),
        line_start: 1,
        line_end: 8,
        context_cost: 8,
        evidence_coverage: score,
        graph_score: score,
        authority_score: score,
        structural_score: score,
        uncertainty: 1.0 - score,
        blocked: false,
        edge_kinds: edge_kinds.iter().map(|kind| (*kind).to_owned()).collect(),
    }
}
