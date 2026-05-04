//! Studio API DTO and Specta type collection exports for the full local runtime.

use specta::TypeCollection;

use super::{
    ApiError, AstSearchResponse, AttachmentSearchResponse, AutocompleteResponse,
    CodeAstAnalysisResponse, DefinitionResolveResponse, DocumentExtractJobStatus,
    DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus, DocumentExtractResult,
    GraphNeighborsResponse, ReferenceSearchResponse, SearchResponse, SymbolSearchResponse,
    Topology3dPayload, UiCapabilities, UiConfig, VfsContentResponse, VfsEntry, VfsScanEntry,
    VfsScanResult,
};

pub use xiuxian_wendao::search::contracts::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind, AstSearchHit,
    AttachmentSearchHit, AutocompleteSuggestion, DefinitionSearchHit, IntentSearchHit,
    KnowledgeSearchHit, MarkdownAnalysisDocumentLink, MarkdownAnalysisDocumentLinkKind,
    MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse, MarkdownRetrievalAtom,
    MermaidProjection, MermaidViewKind, ObservationHint, ReferenceSearchHit, RetrievalChunk,
    RetrievalChunkSurface, SearchBacklinkItem, SearchCorpusIndexStatus, SearchHit,
    SearchIndexMaintenanceStatus, SearchIndexPhase, SearchIndexStatusResponse,
    StudioNavigationTarget,
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
