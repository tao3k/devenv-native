use std::cmp::Ordering;

use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

use super::evidence::{
    candidate_matches_relation_path_evidence, is_policy_authority_markdown_path,
    is_search_strategy_flow_owner_markdown_path, is_validation_authority_markdown_path,
};
use super::path::{is_markdown_path, is_package_source_path, is_test_path, path_has_extension};

pub(super) fn rank_candidate_discovery_results(
    candidates: &mut [SearchStrategyFlowCandidateInput],
) {
    candidates.sort_by(compare_candidate_discovery_results);
}

#[cfg(test)]
pub(super) fn calibrate_candidate_discovery_scores(
    candidates: &mut [SearchStrategyFlowCandidateInput],
) {
    calibrate_candidate_discovery_scores_for_mode(
        candidates,
        CandidateDiscoveryRankingMode::StrategyAuthority,
    );
}

pub(super) fn calibrate_candidate_discovery_scores_for_intent(
    candidates: &mut [SearchStrategyFlowCandidateInput],
    intent: &str,
) {
    calibrate_candidate_discovery_scores_for_mode(candidates, ranking_mode_for_intent(intent));
}

fn calibrate_candidate_discovery_scores_for_mode(
    candidates: &mut [SearchStrategyFlowCandidateInput],
    mode: CandidateDiscoveryRankingMode,
) {
    if mode != CandidateDiscoveryRankingMode::StrategyAuthority {
        return;
    }
    for candidate in candidates {
        match candidate_discovery_priority(candidate) {
            0 => apply_candidate_score_floor(candidate, 0.97, 0.96, 0.97, 0.94, 0.05),
            1 => apply_candidate_score_floor(candidate, 0.95, 0.94, 0.95, 0.92, 0.07),
            2 => apply_candidate_score_floor(candidate, 0.93, 0.92, 0.93, 0.90, 0.08),
            _ => {}
        }
    }
}

pub(super) fn apply_candidate_score_floor(
    candidate: &mut SearchStrategyFlowCandidateInput,
    evidence_coverage: f64,
    graph_score: f64,
    authority_score: f64,
    structural_score: f64,
    uncertainty_ceiling: f64,
) {
    candidate.evidence_coverage = candidate.evidence_coverage.max(evidence_coverage);
    candidate.graph_score = candidate.graph_score.max(graph_score);
    candidate.authority_score = candidate.authority_score.max(authority_score);
    candidate.structural_score = candidate.structural_score.max(structural_score);
    candidate.uncertainty = candidate.uncertainty.min(uncertainty_ceiling);
}

fn compare_candidate_discovery_results(
    left: &SearchStrategyFlowCandidateInput,
    right: &SearchStrategyFlowCandidateInput,
) -> Ordering {
    candidate_discovery_priority(left)
        .cmp(&candidate_discovery_priority(right))
        .then_with(|| compare_score(right.evidence_coverage, left.evidence_coverage))
        .then_with(|| compare_score(right.graph_score, left.graph_score))
        .then_with(|| left.relative_path.cmp(&right.relative_path))
        .then_with(|| left.heading_anchor.cmp(&right.heading_anchor))
}

fn compare_candidate_discovery_corpus_results(
    left: &SearchStrategyFlowCandidateInput,
    right: &SearchStrategyFlowCandidateInput,
) -> Ordering {
    candidate_discovery_corpus_priority(left)
        .cmp(&candidate_discovery_corpus_priority(right))
        .then_with(|| compare_score(right.evidence_coverage, left.evidence_coverage))
        .then_with(|| compare_score(right.graph_score, left.graph_score))
        .then_with(|| compare_score(right.authority_score, left.authority_score))
        .then_with(|| left.relative_path.cmp(&right.relative_path))
        .then_with(|| left.heading_anchor.cmp(&right.heading_anchor))
}

fn compare_score(left: f64, right: f64) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CandidateDiscoveryRankingMode {
    StrategyAuthority,
    CorpusRecall,
}

pub(super) fn ranking_mode_for_intent(intent: &str) -> CandidateDiscoveryRankingMode {
    let normalized = intent.to_ascii_lowercase();
    let mentions_strategy_flow = normalized.contains("searchstrategyflow")
        || (normalized.contains("search")
            && normalized.contains("strategy")
            && normalized.contains("flow"));
    let asks_for_required_evidence = normalized.contains("ownership")
        || normalized.contains("authority")
        || normalized.contains("validation")
        || normalized.contains("relation")
        || normalized.contains("boundary")
        || normalized.contains("page index")
        || normalized.contains("pageindex")
        || normalized.contains("link graph")
        || normalized.contains("linkgraph");
    if mentions_strategy_flow && asks_for_required_evidence {
        CandidateDiscoveryRankingMode::StrategyAuthority
    } else {
        CandidateDiscoveryRankingMode::CorpusRecall
    }
}

pub(super) fn candidate_discovery_priority(candidate: &SearchStrategyFlowCandidateInput) -> u8 {
    let path = candidate.relative_path.to_ascii_lowercase();
    let title = candidate.title.to_ascii_lowercase();
    let combined = format!("{path} {title}");
    if is_search_strategy_flow_owner_markdown_path(path.as_str()) {
        return 0;
    }
    if is_validation_authority_markdown_path(path.as_str(), combined.as_str()) {
        return 1;
    }
    if candidate_matches_relation_path_evidence(candidate) {
        return 2;
    }
    if is_policy_authority_markdown_path(path.as_str(), combined.as_str()) {
        return 3;
    }
    if is_test_path(path.as_str()) {
        return 6;
    }
    if is_markdown_path(path.as_str()) {
        return 4;
    }
    if path_has_extension(path.as_str(), "toml") {
        return 5;
    }
    if is_package_source_path(path.as_str()) {
        return 5;
    }
    6
}

fn candidate_discovery_corpus_priority(candidate: &SearchStrategyFlowCandidateInput) -> u8 {
    let path = candidate.relative_path.to_ascii_lowercase();
    if is_markdown_path(path.as_str()) {
        return 0;
    }
    if path_has_extension(path.as_str(), "toml") {
        return 1;
    }
    if is_package_source_path(path.as_str()) {
        return 2;
    }
    if is_test_path(path.as_str()) {
        return 3;
    }
    4
}

pub(super) fn rank_candidate_discovery_results_for_intent(
    candidates: &mut [SearchStrategyFlowCandidateInput],
    intent: &str,
) {
    match ranking_mode_for_intent(intent) {
        CandidateDiscoveryRankingMode::StrategyAuthority => {
            rank_candidate_discovery_results(candidates);
        }
        CandidateDiscoveryRankingMode::CorpusRecall => {
            candidates.sort_by(compare_candidate_discovery_corpus_results);
        }
    }
}
