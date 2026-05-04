//! Studio API DTO and Specta type collection exports for the full local runtime.

use specta::TypeCollection;

use super::{
    ApiError, DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResult, UiCapabilities, UiConfig, VfsContentResponse, VfsEntry, VfsScanEntry,
    VfsScanResult,
};

pub use xiuxian_wendao::search::contracts::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind, AstSearchHit,
    AstSearchResponse, AttachmentSearchHit, AttachmentSearchResponse, AutocompleteHit,
    AutocompleteResponse, AutocompleteSuggestion, CodeAstAnalysisResponse, CodeAstEdge,
    CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind, CodeAstProjection, CodeAstProjectionKind,
    CodeAstRetrievalAtom, CodeAstRetrievalAtomScope, DefinitionResolveResponse,
    DefinitionSearchHit, GraphLink, GraphNeighborsResponse, GraphNode, IntentSearchHit,
    KnowledgeSearchHit, MarkdownAnalysisDocumentLink, MarkdownAnalysisDocumentLinkKind,
    MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse, MarkdownRetrievalAtom,
    MermaidProjection, MermaidViewKind, ObservationHint, ReferenceSearchHit,
    ReferenceSearchResponse, RetrievalChunk, RetrievalChunkSurface, SearchBacklinkItem,
    SearchCorpusIndexStatus, SearchHit, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse, SearchResponse, StudioNavigationTarget, SymbolSearchHit,
    SymbolSearchResponse, Topology3dPayload, TopologyCluster, TopologyLink, TopologyNode,
};

/// Build the frontend-facing Studio Specta type collection.
#[must_use]
pub fn studio_frontend_type_collection() -> TypeCollection {
    TypeCollection::default()
        .register::<ApiError>()
        .register::<VfsEntry>()
        .register::<VfsScanEntry>()
        .register::<VfsScanResult>()
        .register::<VfsContentResponse>()
        .register::<UiCapabilities>()
        .register::<UiConfig>()
        .register::<GraphNeighborsResponse>()
        .register::<Topology3dPayload>()
        .register::<SearchResponse>()
        .register::<AttachmentSearchResponse>()
        .register::<AstSearchResponse>()
        .register::<DefinitionResolveResponse>()
        .register::<ReferenceSearchResponse>()
        .register::<SymbolSearchResponse>()
        .register::<AutocompleteResponse>()
        .register::<MarkdownAnalysisResponse>()
        .register::<CodeAstAnalysisResponse>()
        .register::<DocumentExtractResult>()
        .register::<DocumentExtractJobSubmitRequest>()
        .register::<DocumentExtractJobStatus>()
        .register::<DocumentExtractJobsStatus>()
}
