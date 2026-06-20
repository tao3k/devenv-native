//! Studio API types exposed through the server crate.

pub use crate::contracts::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind, ApiError,
    AttachmentSearchHit, AttachmentSearchResponse, AutocompleteHit, AutocompleteResponse,
    AutocompleteSuggestion, DefinitionResolveResponse, DefinitionSearchHit,
    DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResource, DocumentExtractResult, GraphLink, GraphNeighborsResponse, GraphNode,
    IntentSearchHit, KnowledgeSearchHit, MarkdownAnalysisDocumentLink,
    MarkdownAnalysisDocumentLinkKind, MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse,
    MarkdownRetrievalAtom, MermaidProjection, MermaidViewKind, ObservationHint, ReferenceSearchHit,
    ReferenceSearchResponse, RetrievalChunk, RetrievalChunkSurface, SearchBacklinkItem,
    SearchCorpusIndexStatus, SearchHit, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse, SearchResponse, SourceSymbolHit, StudioNavigationTarget,
    SymbolSearchHit, SymbolSearchResponse, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode, UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample,
    UiCodeSearchRoutes, UiConfig, UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind,
    UiProjectConfig, UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract, UiRepoProjectConfig,
    UiSearchContract, UiSearchContractAlias, VfsCategory, VfsContentResponse, VfsEntry,
    VfsScanEntry, VfsScanResult, studio_frontend_type_collection, studio_type_collection,
};
