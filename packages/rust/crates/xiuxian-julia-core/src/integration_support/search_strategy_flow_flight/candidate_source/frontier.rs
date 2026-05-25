use std::collections::HashSet;

use crate::integration_support::search_strategy_flow_candidates::SearchStrategyFlowCandidateInput;

use super::evidence::{
    CandidateDiscoveryRequiredEvidence, candidate_matches_ownership_boundary_evidence,
    candidate_matches_relation_path_evidence, candidate_matches_validation_path_evidence,
};
use crate::integration_support::search_strategy_flow_flight::constants::{
    MAX_FLIGHT_DISCOVERY_CANDIDATES, MAX_FLIGHT_REQUIRED_EVIDENCE_FRONTIER_CANDIDATES,
    MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP,
    MIN_FLIGHT_REQUIRED_EVIDENCE_CANDIDATES_BEFORE_EARLY_STOP,
    MIN_FLIGHT_REQUIRED_EVIDENCE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP,
};

pub(super) fn retain_unique_candidate_sources(
    candidates: &mut Vec<SearchStrategyFlowCandidateInput>,
) {
    let mut seen_paths = HashSet::<String>::new();
    candidates.retain(|candidate| seen_paths.insert(candidate.relative_path.clone()));
}

pub(super) fn retain_required_evidence_frontier(
    candidates: &mut Vec<SearchStrategyFlowCandidateInput>,
    required_evidence: CandidateDiscoveryRequiredEvidence,
) {
    if !required_evidence.has_required_bucket() || !required_evidence.is_covered_by(candidates) {
        return;
    }

    let limit = required_evidence
        .min_candidate_count()
        .min(MAX_FLIGHT_REQUIRED_EVIDENCE_FRONTIER_CANDIDATES)
        .max(required_evidence.bucket_count());
    let mut selected = Vec::with_capacity(limit);
    let mut selected_paths = HashSet::<String>::new();

    if required_evidence.ownership_boundary {
        retain_first_matching_candidate(
            candidates,
            &mut selected,
            &mut selected_paths,
            candidate_matches_ownership_boundary_evidence,
        );
    }
    if required_evidence.validation_path {
        retain_first_matching_candidate(
            candidates,
            &mut selected,
            &mut selected_paths,
            candidate_matches_validation_path_evidence,
        );
    }
    if required_evidence.relation_path {
        retain_first_matching_candidate(
            candidates,
            &mut selected,
            &mut selected_paths,
            candidate_matches_relation_path_evidence,
        );
    }

    for candidate in candidates.iter() {
        if selected.len() >= limit {
            break;
        }
        if selected_paths.insert(candidate.relative_path.clone()) {
            selected.push(candidate.clone());
        }
    }
    *candidates = selected;
}

fn retain_first_matching_candidate(
    candidates: &[SearchStrategyFlowCandidateInput],
    selected: &mut Vec<SearchStrategyFlowCandidateInput>,
    selected_paths: &mut HashSet<String>,
    predicate: fn(&SearchStrategyFlowCandidateInput) -> bool,
) {
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| !candidate.blocked && predicate(candidate))
    else {
        return;
    };
    if selected_paths.insert(candidate.relative_path.clone()) {
        selected.push(candidate.clone());
    }
}

pub(super) fn should_stop_candidate_discovery(
    attempted_count: usize,
    candidates: &[SearchStrategyFlowCandidateInput],
    required_evidence: CandidateDiscoveryRequiredEvidence,
) -> bool {
    let covers_relation = candidates
        .iter()
        .any(candidate_matches_relation_path_evidence);
    if required_evidence.has_required_bucket()
        && attempted_count >= 6
        && unique_candidate_source_count(candidates) >= required_evidence.min_candidate_count()
        && required_evidence.is_covered_by(candidates)
    {
        return true;
    }
    if attempted_count >= MIN_FLIGHT_REQUIRED_EVIDENCE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP
        && unique_candidate_source_count(candidates)
            >= MIN_FLIGHT_REQUIRED_EVIDENCE_CANDIDATES_BEFORE_EARLY_STOP
        && covers_relation
        && CandidateDiscoveryRequiredEvidence::all().is_covered_by(candidates)
    {
        return true;
    }

    covers_relation
        && attempted_count >= MIN_FLIGHT_CANDIDATE_DISCOVERY_ATTEMPTS_BEFORE_EARLY_STOP
        && unique_candidate_source_count(candidates) >= MAX_FLIGHT_DISCOVERY_CANDIDATES
}

fn unique_candidate_source_count(candidates: &[SearchStrategyFlowCandidateInput]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.relative_path.as_str())
        .collect::<HashSet<_>>()
        .len()
}
