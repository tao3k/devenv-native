//! Studio API types for TypeScript bindings and HTTP endpoints.
//!
//! This module defines all types used by the Qianji Studio frontend API,
//! including VFS operations, graph queries, search, and UI configuration.

mod analysis;
mod attachments;
mod code_ast;
mod collection;
mod config;
mod definitions;
mod error;
mod graph;
mod navigation;
mod pdf_extract;
mod retrieval;
mod search;
mod search_index;
mod symbols;
mod vfs;

pub use analysis::{
    AnalysisEdge, AnalysisEdgeKind, AnalysisEvidence, AnalysisNode, AnalysisNodeKind,
    MarkdownAnalysisDocumentLink, MarkdownAnalysisDocumentLinkKind,
    MarkdownAnalysisDocumentMetadata, MarkdownAnalysisResponse, MarkdownRetrievalAtom,
    MermaidProjection, MermaidViewKind,
};
pub use attachments::{AttachmentSearchHit, AttachmentSearchResponse};
pub use code_ast::{
    CodeAstAnalysisResponse, CodeAstEdge, CodeAstEdgeKind, CodeAstNode, CodeAstNodeKind,
    CodeAstProjection, CodeAstProjectionKind, CodeAstRetrievalAtom, CodeAstRetrievalAtomScope,
};
pub use collection::{studio_frontend_type_collection, studio_type_collection};
pub use config::{
    UiCapabilities, UiCodeSearchContract, UiCodeSearchContractExample, UiCodeSearchRoutes,
    UiConfig, UiPluginArtifact, UiPluginLaunchSpec, UiPluginTransportKind, UiProjectConfig,
    UiRepoDiscoveryContract, UiRepoDiscoverySurfaceContract, UiRepoProjectConfig, UiSearchContract,
    UiSearchContractAlias,
};
pub use definitions::{
    AstSearchHit, AstSearchResponse, DefinitionResolveResponse, DefinitionSearchHit,
    ObservationHint, ReferenceSearchHit, ReferenceSearchResponse,
};
pub use error::ApiError;
pub use graph::{
    GraphLink, GraphNeighborsResponse, GraphNode, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode,
};
pub use navigation::StudioNavigationTarget;
pub use pdf_extract::{PdfExtractResource, PdfExtractResult};
pub use retrieval::{RetrievalChunk, RetrievalChunkSurface};
pub use search::{
    IntentSearchHit, KnowledgeSearchHit, SearchBacklinkItem, SearchHit, SearchResponse,
};
pub use search_index::{
    SearchCorpusIndexStatus, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse,
};
pub use symbols::{
    AutocompleteHit, AutocompleteResponse, AutocompleteSuggestion, SymbolSearchHit,
    SymbolSearchResponse,
};
pub use vfs::{VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult};
