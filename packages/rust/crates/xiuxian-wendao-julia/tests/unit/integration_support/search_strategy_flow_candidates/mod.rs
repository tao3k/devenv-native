pub(crate) use super::FLIGHT_REPO_SEARCH_CANDIDATE_SOURCE;
pub(crate) use super::{
    CODE_INTELLIGENCE_STRUCTURED_CANDIDATE_COUNT, PRIMARY_MARKDOWN_STRUCTURED_CANDIDATE_COUNT,
    REGISTRY_AUTHORITY_STRUCTURED_CANDIDATE_COUNT, RUST_DUCKDB_STRUCTURED_INDEX_BACKEND,
    TOTAL_STRUCTURED_CANDIDATE_COUNT,
};
pub(crate) use super::{
    MARKDOWN_HEADING_CANDIDATE_SOURCE, MAX_CANDIDATES, SearchStrategyFlowRepoSearchHit,
    audit_configured_search_strategy_flow_markdown_corpus,
    configured_search_strategy_flow_markdown_replay_families,
    configured_search_strategy_flow_markdown_replay_families_with_limit,
    discover_search_strategy_flow_candidate_inputs,
    materialized_search_strategy_flow_markdown_replay_families_from_bridge_report,
    search_strategy_flow_candidate_discovery_contract_json,
    search_strategy_flow_candidate_input_batch_from_markdown,
    search_strategy_flow_candidate_input_from_repo_search_hit,
    search_strategy_flow_total_structured_candidate_index_contract,
};
pub(crate) use super::{
    SearchStrategyFlowConfiguredMarkdownCorpusRow, SearchStrategyFlowConfiguredMarkdownReplayFamily,
};

mod corpus;
mod discovery;
mod materialized_bridge;
mod repo_search;
mod structured_index;
