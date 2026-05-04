//! Lightweight Studio contracts that compile without router or local runtime dependencies.

/// Studio-owned HTTP route contracts and route inventory.
pub mod routes;

pub use routes::{RouteContract, WENDAO_GATEWAY_ROUTE_CONTRACTS};

mod plugin_artifact;

pub use plugin_artifact::{
    UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind, studio_type_collection,
};

mod search_manifest;

pub use search_manifest::{
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiProjectConfig, UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract,
    UiRepoProjectConfig, UiSearchContract, UiSearchContractAlias,
};

#[cfg(feature = "local-runtime")]
mod types;

#[cfg(feature = "local-runtime")]
pub use types::{
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
    studio_frontend_type_collection,
};
