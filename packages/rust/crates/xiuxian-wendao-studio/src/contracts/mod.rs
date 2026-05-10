//! Lightweight Studio contracts that compile without router or local runtime dependencies.

/// Studio-owned HTTP route contracts and route inventory.
pub mod routes;

pub use routes::{RouteContract, WENDAO_GATEWAY_ROUTE_CONTRACTS};

mod analysis_nodes;
mod code_ast;
mod document_extract;
mod error;
mod graph;
mod markdown_analysis;
mod navigation;
mod plugin_artifact;
mod retrieval;
mod search_responses;
mod semantic;
mod symbols;
mod type_collection;
mod vfs;

pub use document_extract::{
    DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResource, DocumentExtractResult,
};
pub use error::ApiError;
pub use plugin_artifact::{UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind};
pub use semantic::{
    StudioContractCategory, StudioContractCenterFlag, StudioContractContentType,
    StudioContractDocType, StudioContractId, StudioContractKind, StudioContractMillisecondsI64,
    StudioContractMillisecondsU64, StudioContractMimeType, StudioContractMode,
    StudioContractNodeKind, StudioContractPath, StudioContractRelationType,
    StudioContractSecondsU64, StudioContractSemanticType, StudioContractState,
    StudioContractStatus, StudioContractTag, StudioContractToken, StudioContractUrl,
};
pub use type_collection::{studio_frontend_type_collection, studio_type_collection};
pub use vfs::{VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult};

mod search_manifest;

pub use search_manifest::{
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiProjectConfig, UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract,
    UiRepoProjectConfig, UiSearchContract, UiSearchContractAlias,
};

#[cfg(feature = "local-runtime")]
mod types;

pub use analysis_nodes::{AnalysisNode, AnalysisNodeKind};
pub use code_ast::{
    CodeAstAnalysisResponse, CodeAstEdge, CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind,
    CodeAstProjection, CodeAstProjectionKind, CodeAstRetrievalAtom, CodeAstRetrievalAtomScope,
};

pub use graph::{
    GraphLink, GraphNeighborsResponse, GraphNode, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode,
};

pub use markdown_analysis::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, MarkdownAnalysisDocumentLink,
    MarkdownAnalysisDocumentLinkKind, MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse,
    MarkdownRetrievalAtom, MermaidProjection, MermaidViewKind,
};

pub use navigation::StudioNavigationTarget;
pub use retrieval::{RetrievalChunk, RetrievalChunkSurface};
#[cfg(all(test, feature = "zhenfa-router"))]
pub(crate) use search_responses::domain_ast_hits_for_search_plane;
pub use search_responses::{
    AstSearchHit, AstSearchResponse, AttachmentSearchHit, AttachmentSearchResponse,
    DefinitionResolveResponse, DefinitionSearchHit, IntentSearchHit, KnowledgeSearchHit,
    ObservationHint, ReferenceSearchHit, ReferenceSearchResponse, SearchBacklinkItem, SearchHit,
    SearchResponse,
};

pub use symbols::{
    AutocompleteHit, AutocompleteResponse, AutocompleteSuggestion, SymbolSearchHit,
    SymbolSearchResponse,
};

#[cfg(feature = "local-runtime")]
pub use types::{
    SearchCorpusIndexStatus, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse,
};
