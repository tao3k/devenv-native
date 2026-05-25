//! Candidate discovery branch for `SearchStrategyFlow` Flight materialization.

mod discovery;
mod evidence;
mod exact_seed;
mod frontier;
mod merge;
mod path;
mod ranking;
mod receipt;

pub(crate) use discovery::search_strategy_flow_candidate_input_batch_from_repo_search;

#[cfg(test)]
use evidence::CandidateDiscoveryRequiredEvidence;
#[cfg(test)]
use evidence::candidate_matches_relation_path_evidence;
#[cfg(test)]
use evidence::candidate_matches_validation_path_evidence;
#[cfg(test)]
use exact_seed::{apply_exact_markdown_attempt_score_floor, candidate_from_exact_markdown_attempt};
#[cfg(test)]
use frontier::{
    retain_required_evidence_frontier, retain_unique_candidate_sources,
    should_stop_candidate_discovery,
};
#[cfg(test)]
use merge::merge_candidate_discovery_result;
#[cfg(test)]
use ranking::{
    CandidateDiscoveryRankingMode, calibrate_candidate_discovery_scores,
    candidate_discovery_priority, rank_candidate_discovery_results,
    rank_candidate_discovery_results_for_intent, ranking_mode_for_intent,
};
#[cfg(test)]
use receipt::{candidate_discovery_attempt_receipt, candidate_discovery_receipt};

#[cfg(test)]
#[path = "../../../../tests/unit/integration_support/search_strategy_flow_flight/candidate_source/mod.rs"]
mod candidate_source_unit_tests;
