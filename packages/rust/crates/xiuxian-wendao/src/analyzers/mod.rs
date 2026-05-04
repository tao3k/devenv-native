//! Repo Intelligence common-core contracts and plugin registry.
//!
//! This module defines the initial Wendao-native contracts for repository
//! intelligence. The first landing focuses on:
//!
//! - repository registration metadata
//! - normalized records for repository understanding
//! - query request/response contracts
//! - plugin registration and dispatch boundaries

/// Analysis cache layer for repository intelligence results.
#[path = "cache/mod.rs"]
mod cache;
/// Configuration types for repository registration.
#[path = "config/mod.rs"]
mod config;
/// Error types for repository intelligence operations.
mod errors;
/// Language-specific plugin guidance; plugin-specific public APIs live in the
/// plugin crates.
#[path = "languages/mod.rs"]
mod languages;
/// Plugin trait definitions and analysis context types.
mod plugin;
/// Projection layer for transforming analysis records into consumable outputs.
#[path = "projection/mod.rs"]
mod projection;
/// Query request and response contracts.
#[path = "query/mod.rs"]
mod query;
/// Normalized record types for repository understanding.
mod records;
/// Plugin registry for dynamic analyzer registration.
mod registry;
mod repo_source;
/// Saliency scoring for symbol and module importance.
mod saliency;
/// Analysis orchestration and repository processing services.
#[path = "service/mod.rs"]
mod service;
/// Verification auditing (skeptic) for documentation coverage.
mod skeptic;

#[cfg(feature = "search-runtime")]
pub use cache::build_repository_analysis_cache_key;
#[cfg(feature = "search-runtime")]
pub use cache::load_cached_repository_analysis_for_revision;
#[cfg(feature = "search-runtime")]
pub(crate) use cache::{
    FingerprintMode, RepositoryAnalysisValkeyScope, ValkeyAnalysisCache, analysis_fingerprint_mode,
    change_affects_analysis_identity, plugin_ids_support_semantic_owner_reuse,
    semantic_fingerprint_for_file,
};
pub use cache::{
    RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey, load_cached_repository_analysis,
    load_cached_repository_search_result, store_cached_repository_analysis,
    store_cached_repository_search_result,
};
#[cfg(feature = "search-runtime")]
pub use cache::{
    RepositorySearchArtifacts, load_cached_repository_search_artifacts,
    store_cached_repository_search_artifacts,
};
pub use config::{
    RegisteredRepository, RepoIntelligenceConfig, RepositoryPluginConfig, RepositoryRef,
    RepositoryRefreshPolicy, load_repo_intelligence_config,
};
pub use errors::RepoIntelligenceError;
pub use plugin::{
    AnalysisContext, PluginAnalysisOutput, PluginLinkContext, RepoIntelligencePlugin,
    RepoSourceFile, RepositoryAnalysisOutput,
};
pub use projection::{
    ProjectedMarkdownDocument, ProjectedPageIndexDocument, ProjectedPageIndexNode,
    ProjectedPageIndexSection, ProjectedPageIndexTree, ProjectedPageRecord, ProjectedPageSection,
    ProjectionInputBundle, ProjectionPageKind, ProjectionPageSeed, build_projected_gap_report,
    build_projected_page, build_projected_page_family_cluster, build_projected_page_family_context,
    build_projected_page_family_search, build_projected_page_index_documents,
    build_projected_page_index_node, build_projected_page_index_tree,
    build_projected_page_index_tree_search, build_projected_page_index_trees,
    build_projected_page_navigation, build_projected_page_navigation_search,
    build_projected_page_search, build_projected_pages, build_projected_retrieval,
    build_projected_retrieval_context, build_projected_retrieval_hit, build_projection_inputs,
    render_projected_markdown_documents,
};
pub use query::{
    DocCoverageQuery, DocCoverageResult, DocsFamilyClusterQuery, DocsFamilyClusterResult,
    DocsFamilyContextQuery, DocsFamilyContextResult, DocsFamilySearchQuery, DocsFamilySearchResult,
    DocsMarkdownDocumentsQuery, DocsMarkdownDocumentsResult, DocsNavigationQuery,
    DocsNavigationResult, DocsNavigationSearchQuery, DocsNavigationSearchResult,
    DocsPageIndexDocumentsQuery, DocsPageIndexDocumentsResult, DocsPageIndexNodeQuery,
    DocsPageIndexNodeResult, DocsPageIndexTreeQuery, DocsPageIndexTreeResult,
    DocsPageIndexTreeSearchQuery, DocsPageIndexTreeSearchResult, DocsPageIndexTreesQuery,
    DocsPageIndexTreesResult, DocsPageQuery, DocsPageResult, DocsPlannerItemQuery,
    DocsPlannerItemResult, DocsPlannerQueueGroup, DocsPlannerQueueQuery, DocsPlannerQueueResult,
    DocsPlannerRankHit, DocsPlannerRankQuery, DocsPlannerRankReason, DocsPlannerRankReasonCode,
    DocsPlannerRankResult, DocsPlannerSearchHit, DocsPlannerSearchQuery, DocsPlannerSearchResult,
    DocsPlannerWorksetBalance, DocsPlannerWorksetFamilyBalanceEntry, DocsPlannerWorksetFamilyGroup,
    DocsPlannerWorksetGapKindBalanceEntry, DocsPlannerWorksetGroup, DocsPlannerWorksetQuery,
    DocsPlannerWorksetQuotaHint, DocsPlannerWorksetResult, DocsPlannerWorksetStrategy,
    DocsPlannerWorksetStrategyCode, DocsPlannerWorksetStrategyReason,
    DocsPlannerWorksetStrategyReasonCode, DocsProjectedGapReportQuery,
    DocsProjectedGapReportResult, DocsRetrievalContextQuery, DocsRetrievalContextResult,
    DocsRetrievalHitQuery, DocsRetrievalHitResult, DocsRetrievalQuery, DocsRetrievalResult,
    DocsSearchQuery, DocsSearchResult, ExampleSearchHit, ExampleSearchQuery, ExampleSearchResult,
    ImportSearchHit, ImportSearchQuery, ImportSearchResult, ModuleSearchHit, ModuleSearchQuery,
    ModuleSearchResult, ProjectedGapKind, ProjectedGapRecord, ProjectedGapSummary,
    ProjectedGapSummaryEntry, ProjectedPageFamilyCluster, ProjectedPageFamilyContextEntry,
    ProjectedPageFamilySearchHit, ProjectedPageIndexNodeContext, ProjectedPageIndexNodeHit,
    ProjectedPageNavigationSearchHit, ProjectedRetrievalHit, ProjectedRetrievalHitKind,
    RefineEntityDocRequest, RefineEntityDocResponse, RepoBacklinkItem, RepoOverviewQuery,
    RepoOverviewResult, RepoProjectedGapReportQuery, RepoProjectedGapReportResult,
    RepoProjectedPageFamilyClusterQuery, RepoProjectedPageFamilyClusterResult,
    RepoProjectedPageFamilyContextQuery, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageFamilySearchQuery, RepoProjectedPageFamilySearchResult,
    RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexNodeResult,
    RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult,
    RepoProjectedPageIndexTreeSearchQuery, RepoProjectedPageIndexTreeSearchResult,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPageIndexTreesResult,
    RepoProjectedPageNavigationQuery, RepoProjectedPageNavigationResult,
    RepoProjectedPageNavigationSearchQuery, RepoProjectedPageNavigationSearchResult,
    RepoProjectedPageQuery, RepoProjectedPageResult, RepoProjectedPageSearchQuery,
    RepoProjectedPageSearchResult, RepoProjectedPagesQuery, RepoProjectedPagesResult,
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalContextResult,
    RepoProjectedRetrievalHitQuery, RepoProjectedRetrievalHitResult, RepoProjectedRetrievalQuery,
    RepoProjectedRetrievalResult, RepoSourceKind, RepoSyncDriftState, RepoSyncFreshnessSummary,
    RepoSyncHealthState, RepoSyncLifecycleSummary, RepoSyncMode, RepoSyncQuery, RepoSyncResult,
    RepoSyncRevisionSummary, RepoSyncStalenessState, RepoSyncState, RepoSyncStatusSummary,
    SymbolSearchHit, SymbolSearchQuery, SymbolSearchResult,
};
pub use records::{
    DiagnosticRecord, DocRecord, DocTargetRecord, ExampleRecord, ImportKind, ImportRecord,
    ModuleRecord, RelationKind, RelationRecord, RepoSymbolKind, RepositoryRecord, SymbolRecord,
};
pub use registry::PluginRegistry;
pub use repo_source::resolve_registered_repository_source;
pub use saliency::compute_repository_saliency;
#[cfg(all(feature = "zhenfa-router", test))]
pub(crate) use service::DocsDocumentSegmentResult;
#[cfg(feature = "search-runtime")]
pub use service::canonical_import_query_text;
#[cfg(feature = "search-runtime")]
pub use service::{
    CachedRepositoryAnalysis, RepoAnalysisFallbackContract,
    analyze_registered_repository_cached_bundle_with_registry,
    analyze_registered_repository_target_file_with_registry,
    build_repo_projected_page_search_with_artifacts, example_fallback_contract,
    import_fallback_contract, module_fallback_contract, repository_search_artifacts,
    symbol_fallback_contract,
};
pub use service::{
    DOCS_CONTRACT_IDS, DOCS_DOCUMENT_CONTRACT_ID, DOCS_NAVIGATION_CONTRACT_ID,
    DOCS_PAGE_INDEX_TREE_CONTRACT_ID, DOCS_RETRIEVAL_CONTEXT_CONTRACT_ID, DOCS_SEARCH_CONTRACT_ID,
    DocsCapabilityContractAssets, DocsCapabilityContractSnapshot, DocsCliContractSnapshot,
    DocsContractDefaultValue, DocsContractParamSnapshot, DocsDocumentToolArgs,
    DocsHttpContractSnapshot, DocsNavigationOptions, DocsNavigationToolArgs,
    DocsPageIndexTreeToolArgs, DocsRetrievalContextOptions, DocsRetrievalContextToolArgs,
    DocsSearchToolArgs, DocsToolContractSnapshot, DocsToolService, docs_capability_contract_assets,
    docs_capability_contract_snapshot, docs_capability_schema_snapshot,
};
#[cfg(feature = "zhenfa-router")]
pub(crate) use service::{DocsToolRuntime, DocsToolRuntimeHandle};
#[cfg(all(feature = "zhenfa-router", feature = "julia"))]
pub(crate) use service::{
    IncrementalApplyContext, analyze_changed_files, apply_incremental_plugin_outputs,
};
#[cfg(feature = "runtime-transport")]
pub use service::{
    JULIA_ARROW_ANALYZER_SCORE_COLUMN, JULIA_ARROW_DOC_ID_COLUMN, JULIA_ARROW_EMBEDDING_COLUMN,
    JULIA_ARROW_FINAL_SCORE_COLUMN, JULIA_ARROW_QUERY_EMBEDDING_COLUMN,
    JULIA_ARROW_TRACE_ID_COLUMN, JULIA_ARROW_VECTOR_SCORE_COLUMN, julia_arrow_request_schema,
    julia_arrow_response_schema,
};
pub use service::{
    analyze_registered_repository, analyze_registered_repository_cached_with_registry,
    analyze_registered_repository_with_registry, analyze_repository_from_config,
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry, build_doc_coverage,
    build_docs_family_cluster, build_docs_family_context, build_docs_family_search,
    build_docs_markdown_documents, build_docs_navigation, build_docs_navigation_search,
    build_docs_page, build_docs_page_index_documents, build_docs_page_index_node,
    build_docs_page_index_tree, build_docs_page_index_tree_search, build_docs_page_index_trees,
    build_docs_planner_item, build_docs_planner_queue, build_docs_planner_rank,
    build_docs_planner_search, build_docs_planner_workset, build_docs_projected_gap_report,
    build_docs_retrieval, build_docs_retrieval_context, build_docs_retrieval_hit,
    build_docs_search, build_example_search, build_import_search, build_module_search,
    build_repo_overview, build_repo_projected_gap_report, build_repo_projected_page,
    build_repo_projected_page_family_cluster, build_repo_projected_page_family_context,
    build_repo_projected_page_family_search, build_repo_projected_page_index_node,
    build_repo_projected_page_index_tree, build_repo_projected_page_index_tree_search,
    build_repo_projected_page_index_trees, build_repo_projected_page_navigation,
    build_repo_projected_page_navigation_search, build_repo_projected_page_search,
    build_repo_projected_pages, build_repo_projected_retrieval,
    build_repo_projected_retrieval_context, build_repo_projected_retrieval_hit,
    build_symbol_search, doc_coverage_from_config, doc_coverage_from_config_with_registry,
    docs_family_cluster_from_config, docs_family_cluster_from_config_with_registry,
    docs_family_context_from_config, docs_family_context_from_config_with_registry,
    docs_family_search_from_config, docs_family_search_from_config_with_registry,
    docs_markdown_documents_from_config, docs_markdown_documents_from_config_with_registry,
    docs_navigation_from_config, docs_navigation_from_config_with_registry,
    docs_navigation_search_from_config, docs_navigation_search_from_config_with_registry,
    docs_page_from_config, docs_page_from_config_with_registry,
    docs_page_index_documents_from_config, docs_page_index_documents_from_config_with_registry,
    docs_page_index_node_from_config, docs_page_index_node_from_config_with_registry,
    docs_page_index_tree_from_config, docs_page_index_tree_from_config_with_registry,
    docs_page_index_tree_search_from_config, docs_page_index_tree_search_from_config_with_registry,
    docs_page_index_trees_from_config, docs_page_index_trees_from_config_with_registry,
    docs_planner_item_from_config, docs_planner_item_from_config_with_registry,
    docs_planner_queue_from_config, docs_planner_queue_from_config_with_registry,
    docs_planner_rank_from_config, docs_planner_rank_from_config_with_registry,
    docs_planner_search_from_config, docs_planner_search_from_config_with_registry,
    docs_planner_workset_from_config, docs_planner_workset_from_config_with_registry,
    docs_projected_gap_report_from_config, docs_projected_gap_report_from_config_with_registry,
    docs_retrieval_context_from_config, docs_retrieval_context_from_config_with_registry,
    docs_retrieval_from_config, docs_retrieval_from_config_with_registry,
    docs_retrieval_hit_from_config, docs_retrieval_hit_from_config_with_registry,
    docs_search_from_config, docs_search_from_config_with_registry, example_search_from_config,
    example_search_from_config_with_registry, import_search_from_config,
    import_search_from_config_with_registry, load_registered_repository, module_search_from_config,
    module_search_from_config_with_registry, repo_overview_from_config,
    repo_overview_from_config_with_registry, repo_projected_gap_report_from_config,
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
    repo_projected_retrieval_hit_from_config_with_registry, repo_sync_for_registered_repository,
    repo_sync_from_config, symbol_search_from_config, symbol_search_from_config_with_registry,
};
#[cfg(any(feature = "search-runtime", feature = "studio"))]
pub(crate) use service::{
    backlinks_for, documents_backlink_lookup, example_match_score, example_relation_lookup,
    hierarchy_segments_from_path, import_match_score, infer_ecosystem, module_match_score,
    normalized_rank_score, projection_page_lookup, projection_pages_for, record_hierarchical_uri,
    related_modules_for_example, related_symbols_for_example, symbol_match_score,
};
pub use skeptic::{AuditResult, audit_symbols};
