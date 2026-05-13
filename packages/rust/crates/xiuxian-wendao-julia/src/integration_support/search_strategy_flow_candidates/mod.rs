//! Candidate surfaces for `WendaoGraph` `SearchStrategyFlow` probes.

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/bridge_report_support.rs"]
mod bridge_report;
mod code_inventory;
#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/corpus_support.rs"]
mod corpus;
mod discovery;
mod offline_audit;
mod registry;
mod repo_search;
mod structured_index;
mod types;

pub(super) use super::search_strategy_flow_evidence_edge_kinds;
#[cfg(test)]
pub(crate) use bridge_report::{
    SearchStrategyFlowMaterializedRepoReplayFamily,
    materialized_search_strategy_flow_markdown_replay_families_from_bridge_report,
};
#[cfg(test)]
pub(crate) use code_inventory::{
    audit_search_strategy_flow_code_intelligence_inventory,
    search_strategy_flow_code_intelligence_inventory_candidate_input_batch,
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
    search_strategy_flow_candidate_input_batch_from_markdown,
    search_strategy_flow_candidate_input_batch_with_discovery_receipt,
};
pub(crate) use offline_audit::link_search_strategy_flow_offline_audit_entrypoints;
#[cfg(test)]
pub(crate) use registry::{
    audit_search_strategy_flow_registry_authority,
    search_strategy_flow_registry_authority_candidate_input_batch,
};
pub(crate) use repo_search::search_strategy_flow_candidate_input_from_repo_search_hit;
pub(crate) use structured_index::{
    REGISTRY_METADATA_CANDIDATE_SOURCE, SearchStrategyFlowStructuredCandidateCounts,
    search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_total_structured_candidate_index_contract_json,
};
#[cfg(test)]
pub(crate) use structured_index::{
    RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
    search_strategy_flow_total_structured_candidate_index_contract,
};
pub(crate) use types::MARKDOWN_HEADING_CANDIDATE_SOURCE;
#[cfg(test)]
pub(crate) use types::MAX_CANDIDATES;
pub(crate) use types::{
    CODE_INTELLIGENCE_CANDIDATE_SOURCE, SearchStrategyFlowCandidateInput,
    SearchStrategyFlowCandidateInputBatch, SearchStrategyFlowRepoSearchHit,
};

#[cfg(test)]
#[path = "../../../tests/unit/integration_support/search_strategy_flow_candidates/mod.rs"]
mod tests;
