//! Search contract types shared by Wendao domain search and Studio adapters.
//!
//! Studio owns HTTP route inventory and TypeScript schema export. Wendao keeps
//! the search payload records that are still used by domain search runtime.

mod analysis;
mod attachments;
mod code_ast;
mod config;
mod definitions;
mod document_extract;
mod error;
mod graph;
#[cfg(feature = "search-runtime")]
mod helpers;
mod navigation;
mod retrieval;
mod search;
#[cfg(feature = "search-runtime")]
#[path = "search_index/mod.rs"]
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
pub use config::{UiProjectConfig, UiRepoProjectConfig};
pub use definitions::{
    AstSearchHit, AstSearchResponse, DefinitionResolveResponse, DefinitionSearchHit,
    ObservationHint, ReferenceSearchHit, ReferenceSearchResponse,
};
pub use document_extract::{
    DocumentExtractJobStatus, DocumentExtractJobSubmitRequest, DocumentExtractJobsStatus,
    DocumentExtractResource, DocumentExtractResult,
};
pub use error::ApiError;
pub use graph::{
    GraphLink, GraphNeighborsResponse, GraphNode, Topology3dPayload, TopologyCluster, TopologyLink,
    TopologyNode,
};
#[cfg(feature = "search-runtime")]
pub(crate) use helpers::{
    SearchProjectMetadata, ast_search_lang, build_code_ast_hits_from_content,
    build_markdown_ast_hits_from_sections, compile_markdown_nodes, configured_project_scopes,
    index_path_for_entry, infer_crate_name, is_markdown_path, markdown_scope_name,
    project_metadata_for_path, resolve_project_root_path, score_reference_hit, should_skip_entry,
};
pub use navigation::StudioNavigationTarget;
pub use retrieval::{RetrievalChunk, RetrievalChunkSurface};
pub use search::{
    IntentSearchHit, KnowledgeSearchHit, SearchBacklinkItem, SearchHit, SearchResponse,
};
#[cfg(feature = "search-runtime")]
pub use search_index::{
    SearchCorpusIndexStatus, SearchIndexMaintenanceStatus, SearchIndexPhase,
    SearchIndexStatusResponse,
};
pub use symbols::{
    AutocompleteHit, AutocompleteResponse, AutocompleteSuggestion, SymbolSearchHit,
    SymbolSearchResponse,
};
pub use vfs::{VfsCategory, VfsContentResponse, VfsEntry, VfsScanEntry, VfsScanResult};
