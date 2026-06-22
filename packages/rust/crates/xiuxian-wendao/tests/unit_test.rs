//! Cargo entry point for xiuxian-wendao unit tests.

#[path = "unit/lib_policy.rs"]
mod lib_policy;

#[path = "unit/episteme/mod.rs"]
mod episteme;

#[path = "unit/repo_config_bridge_audit.rs"]
mod repo_config_bridge_audit;

#[cfg(feature = "performance")]
#[path = "unit/link_graph_perf_support.rs"]
mod link_graph_perf_support;

#[cfg(feature = "search-runtime")]
#[path = "unit/query_core_execute_backends.rs"]
mod query_core_execute_backends;

#[cfg(feature = "search-runtime")]
#[path = "unit/search_contracts_search_index_diagnostics_relations.rs"]
mod search_contracts_search_index_diagnostics_relations;

#[cfg(feature = "search-runtime")]
#[path = "unit/search_repo_content_chunk_schema.rs"]
mod search_repo_content_chunk_schema;
