//! Studio API DTO and Specta type collection exports for the full local runtime.

use specta::TypeCollection;

use super::{UiCapabilities, UiConfig, UiPluginArtifact, UiPluginLaunchSpec};

pub use xiuxian_wendao::search::contracts::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind, ApiError,
    AstSearchHit, AstSearchResponse, AttachmentSearchHit, AttachmentSearchResponse,
    AutocompleteHit, AutocompleteResponse, AutocompleteSuggestion, CodeAstAnalysisResponse,
    CodeAstEdge, CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind, CodeAstProjection,
    CodeAstProjectionKind, CodeAstRetrievalAtom, CodeAstRetrievalAtomScope,
    DefinitionResolveResponse, DefinitionSearchHit, DocumentExtractJobStatus,
    DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus, DocumentExtractResource,
    DocumentExtractResult, GraphLink, GraphNeighborsResponse, GraphNode, IntentSearchHit,
    KnowledgeSearchHit, MarkdownAnalysisDocumentLink, MarkdownAnalysisDocumentLinkKind,
    MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse, MarkdownRetrievalAtom,
    MermaidProjection, MermaidViewKind, ObservationHint, ReferenceSearchHit,
    ReferenceSearchResponse, RetrievalChunk, RetrievalChunkSurface, SearchBacklinkItem,
    SearchCorpusIndexStatus, SearchHit, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse, SearchResponse, StudioNavigationTarget, SymbolSearchHit,
    SymbolSearchResponse, Topology3dPayload, TopologyCluster, TopologyLink, TopologyNode,
    VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult,
};

/// Build the plugin-only Studio Specta type collection.
#[must_use]
pub fn studio_type_collection() -> TypeCollection {
    TypeCollection::default()
        .register::<UiPluginArtifact>()
        .register::<UiPluginLaunchSpec>()
}

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
