//! Markdown link graph index + retrieval algorithms.

#[path = "addressing/mod.rs"]
pub mod addressing;
#[path = "agentic/mod.rs"]
pub mod agentic;
#[path = "context_snapshot/mod.rs"]
mod context_snapshot;
#[path = "index/mod.rs"]
mod index;
#[path = "models/mod.rs"]
mod models;
#[path = "narrator.rs"]
mod narrator;
#[path = "page_index/mod.rs"]
mod page_index;
/// Performance-only `LinkGraph` probes and benchmark helpers.
#[cfg(feature = "performance")]
#[path = "perf_support.rs"]
pub mod perf_support;
#[path = "ppr_hybrid.rs"]
pub mod ppr_hybrid;
#[path = "runtime_config/mod.rs"]
pub(crate) mod runtime_config;
/// `GraphMem` saliency models, scoring, and Valkey persistence adapters.
#[path = "saliency/mod.rs"]
pub mod saliency;
#[path = "stats_cache/mod.rs"]
mod stats_cache;

pub use addressing::{
    Address, EnhancedResolvedNode, IdCollision, IndexedNode, MatchType, ModificationError,
    ModificationResult, PathEntry, PathMatch, RegistryBuildResult, RegistryIndex, ResolveError,
    ResolveMode, ResolvedNode, SkeletonRerankOptions, SkeletonValidatedHit, StructuralTransaction,
    StructuralTransactionCoordinator, StructureUpdateSignal, TopologyIndex, adjust_line_range,
    build_hash_index, build_id_index, replace_byte_range, resolve_node, resolve_with_indices,
    skeleton_rerank, update_section_content,
};

pub use agentic::{
    LINK_GRAPH_SUGGESTED_LINK_DECISION_SCHEMA_VERSION, LINK_GRAPH_SUGGESTED_LINK_SCHEMA_VERSION,
    LinkGraphAgenticCandidatePair, LinkGraphAgenticExecutionConfig,
    LinkGraphAgenticExecutionResult, LinkGraphAgenticExpansionConfig,
    LinkGraphAgenticExpansionPlan, LinkGraphAgenticWorkerExecution, LinkGraphAgenticWorkerPhase,
    LinkGraphAgenticWorkerPlan, LinkGraphSuggestedLink, LinkGraphSuggestedLinkDecision,
    LinkGraphSuggestedLinkDecisionRequest, LinkGraphSuggestedLinkDecisionResult,
    LinkGraphSuggestedLinkRequest, LinkGraphSuggestedLinkState, valkey_suggested_link_decide,
    valkey_suggested_link_decide_with_valkey, valkey_suggested_link_decisions_recent,
    valkey_suggested_link_decisions_recent_with_valkey, valkey_suggested_link_log,
    valkey_suggested_link_log_with_valkey, valkey_suggested_link_recent,
    valkey_suggested_link_recent_latest, valkey_suggested_link_recent_latest_with_valkey,
    valkey_suggested_link_recent_with_valkey,
};
pub use context_snapshot::{
    LINK_GRAPH_QUANTUM_CONTEXT_SNAPSHOT_SCHEMA_VERSION, QuantumContextSnapshot,
    quantum_context_snapshot_id, valkey_quantum_context_snapshot_drop,
    valkey_quantum_context_snapshot_get, valkey_quantum_context_snapshot_get_with_valkey,
    valkey_quantum_context_snapshot_rollback, valkey_quantum_context_snapshot_rollback_with_valkey,
    valkey_quantum_context_snapshot_save, valkey_quantum_context_snapshot_save_with_valkey,
};
pub(crate) use index::IndexedSection;
#[cfg(feature = "vector-store")]
pub use index::search::quantum_fusion::openai_ignition::{
    OpenAiCompatibleSemanticIgnition, OpenAiCompatibleSemanticIgnitionError,
};
#[cfg(feature = "vector-store")]
pub use index::search::quantum_fusion::scoring::{
    BatchQuantumScorer, BatchQuantumScorerError, QUANTUM_SALIENCY_COLUMN,
};
#[cfg(feature = "vector-store")]
pub use index::search::quantum_fusion::vector_ignition::VectorStoreSemanticIgnition;
pub use index::{LinkGraphCacheBuildMeta, LinkGraphIndex, LinkGraphRefreshMode, SymbolRef};
#[cfg(feature = "vector-store")]
pub use index::{
    QuantumContextBuildError, QuantumSemanticIgnition, QuantumSemanticIgnitionError,
    QuantumSemanticIgnitionFuture,
};
pub use models::{
    CognitiveTraceRecord, LINK_GRAPH_POLICY_REASON_VOCAB, LINK_GRAPH_REASON_BACKEND_UNAVAILABLE,
    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT, LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN, LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED, LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY,
    LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT, LINK_GRAPH_REASON_GRAPH_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT, LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_SUFFICIENT, LINK_GRAPH_REASON_HYBRID_SELECTED,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED, LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION,
    LinkGraphAttachment, LinkGraphAttachmentHit, LinkGraphAttachmentKind, LinkGraphCcsAudit,
    LinkGraphConfidenceLevel, LinkGraphDirection, LinkGraphDisplayHit, LinkGraphDocument,
    LinkGraphEdgeType, LinkGraphHit, LinkGraphJuliaRerankTelemetry, LinkGraphLinkFilter,
    LinkGraphMatchStrategy, LinkGraphMetadata, LinkGraphNeighbor, LinkGraphPassage,
    LinkGraphPlannedSearchPayload, LinkGraphPprSubgraphMode, LinkGraphPromotedOverlayTelemetry,
    LinkGraphRelatedFilter, LinkGraphRelatedPprDiagnostics, LinkGraphRelatedPprOptions,
    LinkGraphRetrievalBudget, LinkGraphRetrievalMode, LinkGraphRetrievalPlanInput,
    LinkGraphRetrievalPlanRecord, LinkGraphScope, LinkGraphSearchFilters, LinkGraphSearchOptions,
    LinkGraphSemanticDocument, LinkGraphSemanticDocumentKind, LinkGraphSemanticDocumentScope,
    LinkGraphSemanticIgnitionTelemetry, LinkGraphSemanticSearchPolicy, LinkGraphSortField,
    LinkGraphSortOrder, LinkGraphSortTerm, LinkGraphStats, LinkGraphTagFilter, MarkdownBlock,
    MarkdownBlockKind, PageIndexMeta, PageIndexNode, QuantumAnchorHit, QuantumContext,
    QuantumFusionOptions, QuantumFusionTelemetry, QuantumSemanticSearchRequest,
};
pub use narrator::narrate_subgraph;
pub(crate) use page_index::{
    DEFAULT_PAGE_INDEX_THINNING_TOKEN_THRESHOLD, build_page_index_tree, thin_page_index_tree,
};
pub use runtime_config::{
    LinkGraphIndexRuntimeConfig, LinkGraphRerankFlightRuntimeSettings,
    clear_link_graph_config_home_override, clear_link_graph_wendao_config_override,
    resolve_link_graph_index_runtime, resolve_link_graph_rerank_binding,
    resolve_link_graph_rerank_flight_runtime_settings, resolve_link_graph_rerank_schema_version,
    resolve_link_graph_rerank_score_weights, set_link_graph_config_home_override,
    set_link_graph_wendao_config_override,
};
pub use saliency::{
    LINK_GRAPH_SALIENCY_SCHEMA_VERSION, LinkGraphSaliencyDecaySweepRequest,
    LinkGraphSaliencyDecaySweepResult, LinkGraphSaliencyPolicy, LinkGraphSaliencyState,
    LinkGraphSaliencyTouchRequest, compute_link_graph_saliency, valkey_saliency_decay_all,
    valkey_saliency_decay_all_with_valkey, valkey_saliency_del, valkey_saliency_get,
    valkey_saliency_get_many, valkey_saliency_get_many_with_valkey,
    valkey_saliency_get_with_valkey, valkey_saliency_touch, valkey_saliency_touch_with_valkey,
};
pub use stats_cache::{
    LINK_GRAPH_STATS_CACHE_SCHEMA_VERSION, valkey_stats_cache_del, valkey_stats_cache_get,
    valkey_stats_cache_set,
};
