//! Studio API types exposed through the web crate.

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
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind, UiProjectConfig,
    UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract, UiRepoProjectConfig, UiSearchContract,
    UiSearchContractAlias, VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult,
    studio_frontend_type_collection, studio_type_collection,
};
