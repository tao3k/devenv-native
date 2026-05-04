//! xiuxian-wendao - Wendao business domain crate.
//!
//! Package split rule:
//! - `xiuxian-wendao-core` owns stable shared contracts
//! - `xiuxian-wendao-runtime` owns generic host behavior
//! - `xiuxian-wendao` owns Wendao graph, retrieval, storage, analyzers, and
//!   business-domain services
//!
//! Some host-facing seams still live here during the migration, but new stable
//! contract records should move toward `core`, and new deployment-dependent
//! helpers should move toward `runtime`.
//!
//! Module layout (by domain):
//! - `types` / `knowledge_py`: Knowledge entries and categories
//! - `storage` / `storage_py`: Valkey-backed persistence
//! - `sync` / `sync_py`: Incremental file sync engine
//! - `entity` / `graph` / `graph_py`: Knowledge graph (entities, relations, search)
//! - `enhancer`: `LinkGraph` note enhancement
//! - `link_graph_refs`: `LinkGraph` entity references
//! - `dependency_indexer` / `dep_indexer_py`: Dependency scanning
//! - `unified_symbol` / `unified_symbol_py`: Cross-language symbol index
//! - `parsers`: Canonical Wendao parser families
//!
//! The Python binding implementation lives under `pybindings/` and is
//! exposed through the `xiuxian_wendao::pybindings` namespace when the
//! `pybindings` feature is enabled.
//!
//! Python bindings are available behind the `pybindings` feature so the
//! default build stays free of PyO3-only dependencies.
//!
//! # Examples
//!
//! ```rust
//! use xiuxian_wendao::{KnowledgeEntry, KnowledgeCategory};
//!
//! let entry = KnowledgeEntry::new(
//!     "test-001".to_string(),
//!     "Error Handling Pattern".to_string(),
//!     "Best practices for error handling...".to_string(),
//!     KnowledgeCategory::Pattern,
//! ).with_tags(vec!["error".to_string(), "exception".to_string()]);
//! ```
//!
//! # Knowledge Graph Examples
//!
//! ```rust
//! use xiuxian_wendao::{Entity, Relation, EntityType, RelationType, KnowledgeGraph};
//!
//! let graph = KnowledgeGraph::new();
//!
//! let entity = Entity::new(
//!     "tool:claude-code".to_string(),
//!     "Claude Code".to_string(),
//!     EntityType::Tool,
//!     "AI coding assistant".to_string(),
//! );
//!
//! graph.add_entity(entity).unwrap();
//! ```
// ---------------------------------------------------------------------------
// Core domain modules
// ---------------------------------------------------------------------------
pub mod entity;
pub mod graph;
/// HMAS blackboard protocol contracts and validators.
pub mod hmas;
pub mod kg_cache;
pub mod link_graph;
/// Feature-gated thin bridge into Julia-owned memory compute surfaces.
#[cfg(feature = "julia")]
pub mod memory;
pub mod parsers;
/// Optional Python binding namespace.
#[cfg(feature = "pybindings")]
pub mod pybindings;
/// Internal query-core skeleton for RFC-driven Wendao execution adapters.
#[cfg(feature = "search-runtime")]
pub mod query_core;
/// Repo-intelligence ingestion runtime and status coordination.
#[cfg(feature = "search-runtime")]
pub mod repo_index;
pub mod schemas;
/// Wendao search infrastructure, corpora, query adapters, and shared primitives.
pub mod search;
pub(crate) mod settings;
pub mod storage;
pub mod sync;
#[cfg(all(
    test,
    any(
        feature = "julia",
        feature = "repo-lexical-index",
        feature = "search-runtime",
        feature = "test-support",
        feature = "zhenfa-router"
    )
))]
#[path = "../tests/unit/support/mod.rs"]
pub(crate) mod test_support;
pub mod types;
/// Shared Valkey client helpers for Wendao runtime integrations.
pub mod valkey_common;

// ---------------------------------------------------------------------------
// Fusion recall boost (Rust computation, Python thin wrapper)
// ---------------------------------------------------------------------------
pub mod fusion;

// ---------------------------------------------------------------------------
// Feature modules (enhancer, link graph refs, dependency, unified symbol)
// ---------------------------------------------------------------------------
pub mod analyzers;
/// Bridges contract-testing findings into Wendao knowledge ingestion payloads.
pub mod contract_feedback;
pub mod dependency_indexer;
/// Bounded local relation-engine seam and DuckDB host bridge.
#[cfg(feature = "search-runtime")]
pub mod duckdb;
pub mod enhancer;
pub mod gateway;
pub mod ingress;
pub mod link_graph_refs;
pub mod skill_runtime;
#[cfg(feature = "repo-lexical-index")]
pub mod unified_symbol;
/// High-level search router for integrating multiple backends.
#[cfg(feature = "zhenfa-router")]
pub mod zhenfa_router;

// ---------------------------------------------------------------------------
// Public re-exports (crate API)
// ---------------------------------------------------------------------------
pub use analyzers::{
    AnalysisContext, DiagnosticRecord, DocCoverageQuery, DocCoverageResult, DocRecord,
    ExampleRecord, ExampleSearchHit, ExampleSearchQuery, ExampleSearchResult, ModuleRecord,
    ModuleSearchHit, ModuleSearchQuery, ModuleSearchResult, PluginAnalysisOutput,
    PluginLinkContext, PluginRegistry, RegisteredRepository, RelationKind, RelationRecord,
    RepoBacklinkItem, RepoIntelligenceConfig, RepoIntelligenceError, RepoIntelligencePlugin,
    RepoOverviewQuery, RepoOverviewResult, RepoSourceFile, RepoSourceKind, RepoSymbolKind,
    RepoSyncDriftState, RepoSyncFreshnessSummary, RepoSyncHealthState, RepoSyncLifecycleSummary,
    RepoSyncMode, RepoSyncQuery, RepoSyncResult, RepoSyncRevisionSummary, RepoSyncStalenessState,
    RepoSyncState, RepoSyncStatusSummary, RepositoryAnalysisOutput, RepositoryPluginConfig,
    RepositoryRecord, RepositoryRef, RepositoryRefreshPolicy, SymbolRecord, SymbolSearchHit,
    SymbolSearchQuery, SymbolSearchResult, analyze_repository_from_config,
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry, build_doc_coverage,
    build_example_search, build_module_search, build_repo_overview, build_symbol_search,
    doc_coverage_from_config, doc_coverage_from_config_with_registry, example_search_from_config,
    example_search_from_config_with_registry, load_registered_repository,
    load_repo_intelligence_config, module_search_from_config,
    module_search_from_config_with_registry, repo_overview_from_config,
    repo_overview_from_config_with_registry, repo_sync_from_config, symbol_search_from_config,
    symbol_search_from_config_with_registry,
};
pub use contract_feedback::WendaoContractFeedbackAdapter;
pub use dependency_indexer::{
    ConfigExternalDependency, DependencyBuildConfig, DependencyConfig, DependencyIndexResult,
    DependencyIndexer, DependencyStats, ExternalSymbol, SymbolIndex, SymbolKind,
};
pub use enhancer::{
    EnhancedNote, EntityRefData, InferredRelation, NoteInput, RefStatsData,
    WendaoResourceLinkTarget, WendaoResourceRegistry, classify_skill_reference, enhance_note,
    enhance_notes_batch, infer_relations,
};
pub use entity::{
    Entity, EntitySearchQuery, EntityType, GraphStats, MultiHopOptions, Relation, RelationType,
};
pub use graph::{KnowledgeGraph, QueryIntent, SkillDoc, SkillRegistrationResult, extract_intent};
pub use hmas::{
    HmasConclusionPayload, HmasDigitalThreadPayload, HmasEvidencePayload, HmasRecordKind,
    HmasSourceNode, HmasTaskPayload, HmasValidationIssue, HmasValidationReport,
    validate_blackboard_file, validate_blackboard_markdown,
};
pub use ingress::{
    ContentHashStore, InMemoryContentHashStore, KnowledgeGraphAssimilationSink,
    NoopPartialReindexHook, PartialReindexHook, SpiderIngressError, SpiderPagePayload,
    SpiderWendaoBridge, WebAssimilationSink, WebIngestionSignal, canonical_web_uri,
    web_namespace_from_url,
};
#[cfg(feature = "vector-store")]
pub use link_graph::{
    BatchQuantumScorer, BatchQuantumScorerError, OpenAiCompatibleSemanticIgnition,
    OpenAiCompatibleSemanticIgnitionError, QUANTUM_SALIENCY_COLUMN, QuantumContextBuildError,
    QuantumSemanticIgnition, QuantumSemanticIgnitionError, QuantumSemanticIgnitionFuture,
    VectorStoreSemanticIgnition,
};
pub use link_graph::{
    LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION, LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION,
    LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION,
    LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION, LinkGraphAgenticCandidatePair,
    LinkGraphAgenticExecutionConfig, LinkGraphAgenticExecutionResult,
    LinkGraphAgenticExpansionConfig, LinkGraphAgenticExpansionPlan,
    LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase, LinkGraphAgenticWorkerPlan,
    LinkGraphAttachment, LinkGraphAttachmentHit, LinkGraphAttachmentKind, LinkGraphCacheBuildMeta,
    LinkGraphConfidenceLevel, LinkGraphDirection, LinkGraphDisplayHit, LinkGraphDocument,
    LinkGraphEdgeType, LinkGraphHit, LinkGraphIndex, LinkGraphLinkFilter, LinkGraphMatchStrategy,
    LinkGraphMetadata, LinkGraphNeighbor, LinkGraphPassage, LinkGraphPlannedSearchPayload,
    LinkGraphPprSubgraphMode, LinkGraphRefreshMode, LinkGraphRelatedFilter,
    LinkGraphRelatedPprDiagnostics, LinkGraphRelatedPprOptions, LinkGraphRetrievalBudget,
    LinkGraphRetrievalMode, LinkGraphRetrievalPlanRecord, LinkGraphSaliencyDecaySweepRequest,
    LinkGraphSaliencyPolicy, LinkGraphSaliencyState, LinkGraphSaliencyTouchRequest, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSemanticIgnitionTelemetry,
    LinkGraphSortField, LinkGraphSortOrder, LinkGraphSortTerm, LinkGraphStats,
    LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision, LinkGraphSuggestedLinkDecisionRequest,
    LinkGraphSuggestedLinkDecisionResult, LinkGraphSuggestedLinkRequest,
    LinkGraphSuggestedLinkState, LinkGraphTagFilter, QuantumAnchorHit, QuantumContext,
    QuantumContextSnapshot, QuantumFusionOptions, QuantumFusionTelemetry,
    QuantumSemanticSearchRequest, clear_link_graph_config_home_override,
    clear_link_graph_wendao_config_override, compute_link_graph_saliency, narrate_subgraph,
    quantum_context_snapshot_id, resolve_link_graph_index_runtime,
    set_link_graph_config_home_override, set_link_graph_wendao_config_override,
    valkey_quantum_context_snapshot_drop, valkey_quantum_context_snapshot_get,
    valkey_quantum_context_snapshot_get_with_valkey, valkey_quantum_context_snapshot_rollback,
    valkey_quantum_context_snapshot_rollback_with_valkey, valkey_quantum_context_snapshot_save,
    valkey_quantum_context_snapshot_save_with_valkey, valkey_saliency_decay_all,
    valkey_saliency_decay_all_with_valkey, valkey_saliency_del, valkey_saliency_get,
    valkey_saliency_get_with_valkey, valkey_saliency_touch, valkey_saliency_touch_with_valkey,
    valkey_suggested_link_decide, valkey_suggested_link_decide_with_valkey,
    valkey_suggested_link_decisions_recent, valkey_suggested_link_decisions_recent_with_valkey,
    valkey_suggested_link_log, valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent,
    valkey_suggested_link_recent_latest, valkey_suggested_link_recent_latest_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};
pub use link_graph_refs::{
    LinkGraphEntityRef, LinkGraphRefStats, count_entity_refs, extract_entity_refs,
    extract_entity_refs_batch, find_notes_referencing_entity, get_ref_stats, is_valid_entity_ref,
    parse_entity_ref,
};
pub use parsers::link_graph::query::{ParsedLinkGraphQuery, parse_search_query};
pub use search::{
    FuzzyMatch, FuzzyMatcher, FuzzyScore, FuzzySearchOptions, LexicalMatcher, edit_distance,
    levenshtein_distance, normalized_score, passes_prefix_requirement, shared_prefix_len,
};
#[cfg(feature = "repo-lexical-index")]
pub use search::{
    SearchDocument, SearchDocumentFields, SearchDocumentIndex, TantivyDocumentMatch, TantivyMatcher,
};
pub use skill_runtime::{
    ATTR_JOURNAL_CARRYOVER, ATTR_TIMER_REMINDED, ATTR_TIMER_SCHEDULED, AssetRequest,
    SkillInventory, SkillInventoryMount, SkillManifest, SkillNamespaceIndex, SkillNamespaceMount,
    SkillRuntimeError, SkillRuntimeResolver, SkillWorkflowType, WendaoAssetHandle,
    ZHIXING_SKILL_DOC_PATH, ZhixingIndexSummary, ZhixingWendaoIndexer,
    build_embedded_wendao_registry, embedded_discover_canonical_uris, embedded_resource_text,
    embedded_resource_text_from_wendao_uri, embedded_skill_links_for_id,
    embedded_skill_links_for_reference_type, embedded_skill_links_index, embedded_skill_markdown,
};
pub use storage::KnowledgeStorage;
pub use sync::{
    DiscoveryOptions, FileChange, IncrementalSyncPolicy, SyncEngine, SyncManifest, SyncResult,
    extract_extensions_from_glob_patterns,
};
pub use types::{KnowledgeCategory, KnowledgeEntry, KnowledgeSearchQuery, KnowledgeStats};
#[cfg(feature = "repo-lexical-index")]
pub use unified_symbol::{SymbolSource, UnifiedIndexStats, UnifiedSymbol, UnifiedSymbolIndex};
pub use xiuxian_wendao_core::{WENDAO_URI_SCHEME, WendaoResourceUri, WendaoResourceUriError};

#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::WendaoZhenfaRouter;
/// Directly execute a search via the router using standard request types.
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::execute_search;
/// Execute a search via the router using raw RPC parameters.
#[cfg(feature = "zhenfa-router")]
pub use zhenfa_router::search_from_rpc_params;

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::wendao_rust_harness_config()
);
