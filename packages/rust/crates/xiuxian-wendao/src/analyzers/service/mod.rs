//! High-level repository intelligence service orchestration.

mod analysis;
mod bootstrap;
mod cached;
mod helpers;
#[cfg(all(feature = "zhenfa-router", feature = "julia"))]
mod incremental;
mod julia_transport;
mod merge;
mod projection;
mod registry;
mod relation_dedupe;
mod search;
mod sync;

#[cfg(feature = "studio")]
pub(crate) use analysis::analyze_registered_repository_target_file_with_registry;
pub use analysis::{
    analyze_registered_repository, analyze_registered_repository_with_registry,
    analyze_repository_from_config, analyze_repository_from_config_with_registry,
};
pub use bootstrap::bootstrap_builtin_registry;
pub use cached::analyze_registered_repository_cached_with_registry;
#[cfg(feature = "studio")]
pub(crate) use cached::{
    CachedRepositoryAnalysis, analyze_registered_repository_cached_bundle_with_registry,
};
#[cfg(any(feature = "search-runtime", feature = "studio"))]
pub(crate) use helpers::{
    backlinks_for, documents_backlink_lookup, example_match_score, example_relation_lookup,
    hierarchy_segments_from_path, infer_ecosystem, module_match_score, projection_page_lookup,
    projection_pages_for, record_hierarchical_uri, related_modules_for_example,
    related_symbols_for_example, symbol_match_score,
};
#[cfg(test)]
pub(crate) use helpers::{
    docs_in_scope, documented_symbol_ids, relation_kind_label, repo_hierarchical_uri,
    resolve_module_scope, symbols_in_scope,
};
pub(crate) use helpers::{import_match_score, normalized_rank_score};
#[cfg(all(feature = "zhenfa-router", feature = "julia"))]
pub(crate) use incremental::{
    IncrementalApplyContext, analyze_changed_files, apply_incremental_plugin_outputs,
};
pub use julia_transport::{
    JULIA_ARROW_ANALYZER_SCORE_COLUMN, JULIA_ARROW_DOC_ID_COLUMN, JULIA_ARROW_EMBEDDING_COLUMN,
    JULIA_ARROW_FINAL_SCORE_COLUMN, JULIA_ARROW_QUERY_EMBEDDING_COLUMN,
    JULIA_ARROW_TRACE_ID_COLUMN, JULIA_ARROW_VECTOR_SCORE_COLUMN, julia_arrow_request_schema,
    julia_arrow_response_schema,
};

#[cfg(all(feature = "zhenfa-router", test))]
pub use projection::DocsDocumentSegmentResult;
#[cfg(feature = "studio")]
pub(crate) use projection::build_repo_projected_page_search_with_artifacts;
pub use projection::{
    DOCS_CONTRACT_IDS, DOCS_DOCUMENT_CONTRACT_ID, DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID, DOCS_SEARCH_CONTRACT_ID,
    DocsCapabilityContractAssets, DocsCapabilityContractSnapshot, DocsCliContractSnapshot,
    DocsContractDefaultValue, DocsContractParamSnapshot, DocsDocumentToolArgs,
    DocsHttpContractSnapshot, DocsNavigationOptions, DocsNavigationToolArgs,
    DocsPageIndexTreeToolArgs, DocsRetrievalContextOptions, DocsRetrievalContextToolArgs,
    DocsSearchToolArgs, DocsToolContractSnapshot, DocsToolService, build_docs_family_cluster,
    build_docs_family_context, build_docs_family_search, build_docs_markdown_documents,
    build_docs_navigation, build_docs_navigation_search, build_docs_page,
    build_docs_page_index_documents, build_docs_page_index_node, build_docs_page_index_tree,
    build_docs_page_index_tree_search, build_docs_page_index_trees, build_docs_planner_item,
    build_docs_planner_queue, build_docs_planner_rank, build_docs_planner_search,
    build_docs_planner_workset, build_docs_projected_gap_report, build_docs_retrieval,
    build_docs_retrieval_context, build_docs_retrieval_hit, build_docs_search,
    build_repo_projected_gap_report, build_repo_projected_page,
    build_repo_projected_page_family_cluster, build_repo_projected_page_family_context,
    build_repo_projected_page_family_search, build_repo_projected_page_index_node,
    build_repo_projected_page_index_tree, build_repo_projected_page_index_tree_search,
    build_repo_projected_page_index_trees, build_repo_projected_page_navigation,
    build_repo_projected_page_navigation_search, build_repo_projected_page_search,
    build_repo_projected_pages, build_repo_projected_retrieval,
    build_repo_projected_retrieval_context, build_repo_projected_retrieval_hit,
    docs_capability_contract_assets, docs_capability_contract_snapshot,
    docs_capability_schema_snapshot, docs_family_cluster_from_config,
    docs_family_cluster_from_config_with_registry, docs_family_context_from_config,
    docs_family_context_from_config_with_registry, docs_family_search_from_config,
    docs_family_search_from_config_with_registry, docs_markdown_documents_from_config,
    docs_markdown_documents_from_config_with_registry, docs_navigation_from_config,
    docs_navigation_from_config_with_registry, docs_navigation_search_from_config,
    docs_navigation_search_from_config_with_registry, docs_page_from_config,
    docs_page_from_config_with_registry, docs_page_index_documents_from_config,
    docs_page_index_documents_from_config_with_registry, docs_page_index_node_from_config,
    docs_page_index_node_from_config_with_registry, docs_page_index_tree_from_config,
    docs_page_index_tree_from_config_with_registry, docs_page_index_tree_search_from_config,
    docs_page_index_tree_search_from_config_with_registry, docs_page_index_trees_from_config,
    docs_page_index_trees_from_config_with_registry, docs_planner_item_from_config,
    docs_planner_item_from_config_with_registry, docs_planner_queue_from_config,
    docs_planner_queue_from_config_with_registry, docs_planner_rank_from_config,
    docs_planner_rank_from_config_with_registry, docs_planner_search_from_config,
    docs_planner_search_from_config_with_registry, docs_planner_workset_from_config,
    docs_planner_workset_from_config_with_registry, docs_projected_gap_report_from_config,
    docs_projected_gap_report_from_config_with_registry, docs_retrieval_context_from_config,
    docs_retrieval_context_from_config_with_registry, docs_retrieval_from_config,
    docs_retrieval_from_config_with_registry, docs_retrieval_hit_from_config,
    docs_retrieval_hit_from_config_with_registry, docs_search_from_config,
    docs_search_from_config_with_registry, repo_projected_gap_report_from_config,
    repo_projected_gap_report_from_config_with_registry,
    repo_projected_page_family_cluster_from_config,
    repo_projected_page_family_cluster_from_config_with_registry,
    repo_projected_page_family_context_from_config,
    repo_projected_page_family_context_from_config_with_registry,
    repo_projected_page_family_search_from_config,
    repo_projected_page_family_search_from_config_with_registry, repo_projected_page_from_config,
    repo_projected_page_from_config_with_registry, repo_projected_page_index_node_from_config,
    repo_projected_page_index_node_from_config_with_registry,
    repo_projected_page_index_tree_from_config,
    repo_projected_page_index_tree_from_config_with_registry,
    repo_projected_page_index_tree_search_from_config,
    repo_projected_page_index_tree_search_from_config_with_registry,
    repo_projected_page_index_trees_from_config,
    repo_projected_page_index_trees_from_config_with_registry,
    repo_projected_page_navigation_from_config,
    repo_projected_page_navigation_from_config_with_registry,
    repo_projected_page_navigation_search_from_config,
    repo_projected_page_navigation_search_from_config_with_registry,
    repo_projected_page_search_from_config, repo_projected_page_search_from_config_with_registry,
    repo_projected_pages_from_config, repo_projected_pages_from_config_with_registry,
    repo_projected_retrieval_context_from_config,
    repo_projected_retrieval_context_from_config_with_registry,
    repo_projected_retrieval_from_config, repo_projected_retrieval_from_config_with_registry,
    repo_projected_retrieval_hit_from_config,
    repo_projected_retrieval_hit_from_config_with_registry,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use projection::{DocsToolRuntime, DocsToolRuntimeHandle};
pub use registry::load_registered_repository;
#[cfg(feature = "studio")]
pub(crate) use search::ExampleSearchMetadata;
#[cfg(feature = "search-runtime")]
pub(crate) use search::canonical_import_query_text;
#[cfg(feature = "studio")]
pub(crate) use search::{
    RepoAnalysisFallbackContract, example_fallback_contract, import_fallback_contract,
    module_fallback_contract, repository_search_artifacts, symbol_fallback_contract,
};
pub use search::{
    build_doc_coverage, build_example_search, build_import_search, build_module_search,
    build_repo_overview, build_symbol_search, doc_coverage_from_config,
    doc_coverage_from_config_with_registry, example_search_from_config,
    example_search_from_config_with_registry, import_search_from_config,
    import_search_from_config_with_registry, module_search_from_config,
    module_search_from_config_with_registry, repo_overview_from_config,
    repo_overview_from_config_with_registry, symbol_search_from_config,
    symbol_search_from_config_with_registry,
};
pub use sync::{repo_sync_for_registered_repository, repo_sync_from_config};
#[cfg(test)]
#[path = "../../../tests/unit/analyzers/service/mod.rs"]
mod tests;
