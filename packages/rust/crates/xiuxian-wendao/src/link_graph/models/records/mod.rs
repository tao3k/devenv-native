mod diagnostics;
mod document;
mod graph_rows;
mod hits;
#[path = "markdown_block/mod.rs"]
mod markdown_block;
mod page_index;
mod passage;
mod payload;
mod quantum_fusion;
mod retrieval_plan;
mod semantic_document;

pub use diagnostics::LinkGraphRelatedPprDiagnostics;
pub use document::LinkGraphDocument;
pub use graph_rows::{LinkGraphMetadata, LinkGraphNeighbor, LinkGraphStats};
pub use hits::{LinkGraphDisplayHit, LinkGraphHit};
pub(crate) use markdown_block::markdown_block_matches_kind;
pub use markdown_block::{BlockAddress, MarkdownBlock, MarkdownBlockKind};
pub use page_index::{PageIndexMeta, PageIndexNode};
pub use passage::LinkGraphPassage;
pub use payload::{
    LinkGraphCcsAudit, LinkGraphJuliaRerankTelemetry, LinkGraphPlannedSearchPayload,
    LinkGraphPromotedOverlayTelemetry, LinkGraphSemanticIgnitionTelemetry,
};
pub use quantum_fusion::{
    QuantumAnchorHit, QuantumContext, QuantumFusionOptions, QuantumFusionTelemetry,
    QuantumSemanticSearchRequest,
};
pub use retrieval_plan::{
    LINK_GRAPH_POLICY_REASON_VOCAB, LINK_GRAPH_REASON_BACKEND_UNAVAILABLE,
    LINK_GRAPH_REASON_GRAPH_INSUFFICIENT, LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_MODE_CONFLICT,
    LINK_GRAPH_REASON_GRAPH_ONLY_PAYLOAD_OVERRIDDEN, LINK_GRAPH_REASON_GRAPH_ONLY_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED, LINK_GRAPH_REASON_GRAPH_ONLY_REQUESTED_EMPTY,
    LINK_GRAPH_REASON_GRAPH_ONLY_SEARCH_TIMEOUT, LINK_GRAPH_REASON_GRAPH_POLICY_MISSING,
    LINK_GRAPH_REASON_GRAPH_POLICY_MODE_CONFLICT, LINK_GRAPH_REASON_GRAPH_SEARCH_TIMEOUT,
    LINK_GRAPH_REASON_GRAPH_SUFFICIENT, LINK_GRAPH_REASON_HYBRID_SELECTED,
    LINK_GRAPH_REASON_VECTOR_ONLY_REQUESTED, LINK_GRAPH_RETRIEVAL_PLAN_SCHEMA_VERSION,
    LinkGraphConfidenceLevel, LinkGraphRetrievalBudget, LinkGraphRetrievalMode,
    LinkGraphRetrievalPlanInput, LinkGraphRetrievalPlanRecord,
};
pub use semantic_document::{
    CognitiveTraceRecord, LinkGraphSemanticDocument, LinkGraphSemanticDocumentKind,
};
