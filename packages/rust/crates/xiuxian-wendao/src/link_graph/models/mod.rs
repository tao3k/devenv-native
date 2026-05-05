//! Shared models for markdown link-graph indexing and retrieval.

mod attachments;
#[path = "query/mod.rs"]
mod query;
#[path = "records/mod.rs"]
mod records;
mod semantic_policy;

pub use attachments::{
    LinkGraphAttachment, LinkGraphAttachmentHit, LinkGraphAttachmentKind, VisionAnnotation,
};
pub use query::{
    LinkGraphDirection, LinkGraphEdgeType, LinkGraphLinkFilter, LinkGraphMatchStrategy,
    LinkGraphPprSubgraphMode, LinkGraphRelatedFilter, LinkGraphRelatedPprOptions, LinkGraphScope,
    LinkGraphSearchFilters, LinkGraphSearchOptions, LinkGraphSortField, LinkGraphSortOrder,
    LinkGraphSortTerm, LinkGraphTagFilter,
};
pub(crate) use records::markdown_block_matches_kind;
pub use records::{
    BlockAddress, CognitiveTraceRecord, LINK_GRAPH_POLICY_REASON_VOCAB,
    LINK_GRAPH_REASON_BACKEND_UNAVAILABLE, LINK_GRAPH_REASON_GRAPH_INSUFFICIENT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN, LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED, LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY,
    LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT, LINK_GRAPH_REASON_GRAPH_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT, LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_SUFFICIENT, LINK_GRAPH_REASON_HYBRID_SELECTED,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED, LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION,
    LinkGraphCcsAudit, LinkGraphConfidenceLevel, LinkGraphDisplayHit, LinkGraphDocument,
    LinkGraphHit, LinkGraphJuliaRerankTelemetry, LinkGraphMetadata, LinkGraphNeighbor,
    LinkGraphPassage, LinkGraphPlannedSearchPayload, LinkGraphPromotedOverlayTelemetry,
    LinkGraphRelatedPprDiagnostics, LinkGraphRetrievalBudget, LinkGraphRetrievalMode,
    LinkGraphRetrievalPlanInput, LinkGraphRetrievalPlanRecord, LinkGraphSemanticDocument,
    LinkGraphSemanticDocumentKind, LinkGraphSemanticIgnitionTelemetry, LinkGraphStats,
    MarkdownBlock, MarkdownBlockKind, PageIndexMeta, PageIndexNode, QuantumAnchorHit,
    QuantumContext, QuantumFusionOptions, QuantumFusionTelemetry, QuantumSemanticSearchRequest,
};
pub use semantic_policy::{LinkGraphSemanticDocumentScope, LinkGraphSemanticSearchPolicy};
