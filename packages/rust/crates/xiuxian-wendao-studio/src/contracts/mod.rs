//! Lightweight Studio contracts that compile without router or local runtime dependencies.

/// Studio-owned HTTP route contracts and route inventory.
pub mod routes;

pub use routes::{RouteContract, WENDAO_GATEWAY_ROUTE_CONTRACTS};

mod document_extract;
mod error;
mod plugin_artifact;
mod type_collection;
mod vfs;

pub use document_extract::{
    DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResource, DocumentExtractResult,
};
pub use error::ApiError;
pub use plugin_artifact::{UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind};
pub use type_collection::studio_type_collection;
pub use vfs::{VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult};

mod search_manifest;

pub use search_manifest::{
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiProjectConfig, UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract,
    UiRepoProjectConfig, UiSearchContract, UiSearchContractAlias,
};

#[cfg(feature = "local-runtime")]
mod code_ast;
#[cfg(feature = "local-runtime")]
mod graph;
#[cfg(feature = "local-runtime")]
mod search_responses;
#[cfg(feature = "local-runtime")]
mod symbols;
#[cfg(feature = "local-runtime")]
mod types;

#[cfg(feature = "local-runtime")]
pub use code_ast::{
    CodeAstAnalysisResponse, CodeAstEdge, CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind,
    CodeAstProjection, CodeAstProjectionKind, CodeAstRetrievalAtom, CodeAstRetrievalAtomScope,
};

#[cfg(feature = "local-runtime")]
pub use graph::{
    GraphLink, GraphNeighborsResponse, GraphNode, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode,
};

#[cfg(feature = "local-runtime")]
pub use search_responses::{
    AstSearchResponse, AttachmentSearchResponse, DefinitionResolveResponse,
    ReferenceSearchResponse, SearchResponse,
};

#[cfg(feature = "local-runtime")]
pub use symbols::{AutocompleteHit, AutocompleteResponse, SymbolSearchHit, SymbolSearchResponse};

#[cfg(feature = "local-runtime")]
pub use types::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind, AstSearchHit,
    AttachmentSearchHit, AutocompleteSuggestion, DefinitionSearchHit, IntentSearchHit,
    KnowledgeSearchHit, MarkdownAnalysisDocumentLink, MarkdownAnalysisDocumentLinkKind,
    MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse, MarkdownRetrievalAtom,
    MermaidProjection, MermaidViewKind, ObservationHint, ReferenceSearchHit, RetrievalChunk,
    RetrievalChunkSurface, SearchBacklinkItem, SearchCorpusIndexStatus, SearchHit,
    SearchIndexMaintenanceStatus, SearchIndexPhase, SearchIndexStatusResponse,
    StudioNavigationTarget, studio_frontend_type_collection,
};
