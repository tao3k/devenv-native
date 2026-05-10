//! Candidate surfaces for `WendaoGraph` `SearchStrategyFlow` probes.

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/bridge_report_support.rs"]
mod bridge_report;
#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/corpus_support.rs"]
mod corpus;
mod discovery;
mod repo_search;
mod types;

pub(super) use super::search_strategy_flow_evidence_edge_kinds;
#[cfg(test)]
pub(crate) use bridge_report::{
    SearchStrategyFlowMaterializedRepoReplayFamily,
    materialized_search_strategy_flow_markdown_replay_families_from_bridge_report,
};
#[cfg(test)]
pub(crate) use corpus::{
    SearchStrategyFlowConfiguredMarkdownCorpusRow,
    SearchStrategyFlowConfiguredMarkdownReplayFamily,
    audit_configured_search_strategy_flow_markdown_corpus,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
};
#[cfg(test)]
pub(crate) use discovery::discover_search_strategy_flow_candidate_inputs;
pub(crate) use discovery::{
    search_strategy_flow_candidate_input_batch,
    search_strategy_flow_candidate_input_batch_from_markdown,
};
pub(crate) use repo_search::search_strategy_flow_candidate_input_from_repo_search_hit;
pub(crate) use types::{
    FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch, SearchStrategyFlowRepoSearchHit,
};
#[cfg(test)]
pub(crate) use types::{MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES};

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/mod.rs"]
mod tests;
