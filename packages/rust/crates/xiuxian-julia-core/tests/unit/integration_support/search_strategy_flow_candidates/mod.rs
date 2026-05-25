pub(crate) use super::CODE_INTELLIGENCE_CANDIDATE_SOURCE;
pub(crate) use super::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES, SearchStrategyFlowRepoSearchHit,
    SearchStrategyFlowStructuredCandidateCounts,
    audit_configured_search_strategy_flow_markdown_corpus,
    audit_search_strategy_flow_code_intelligence_inventory,
    audit_search_strategy_flow_registry_authority,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
    discover_search_strategy_flow_candidate_inputs,
    discover_search_strategy_flow_candidate_inputs_with_limit,
    materialized_search_strategy_flow_markdown_replay_families_from_bridge_report,
    search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_candidate_input_batch_from_markdown,
    search_strategy_flow_candidate_input_from_repo_search_hit,
    search_strategy_flow_candidate_inputs_arrow_ipc,
    search_strategy_flow_candidate_inputs_arrow_record_batch,
    search_strategy_flow_code_intelligence_inventory_candidate_input_batch,
    search_strategy_flow_registry_authority_candidate_input_batch,
    search_strategy_flow_total_structured_candidate_index_contract,
};
pub(crate) use super::{REGISTRY_METADATA_CANDIDATE_SOURCE, RUST_DUCKDB_STRUCTURED_INDEX_BACKEND};
pub(crate) use super::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowConfiguredMarkdownCorpusRow,
    SearchStrategyFlowConfiguredMarkdownReplayFamily,
};

mod arrow_service;
mod code_inventory;
mod corpus;
mod discovery;
mod materialized_bridge;
mod registry;
mod repo_search;
mod structured_index;
