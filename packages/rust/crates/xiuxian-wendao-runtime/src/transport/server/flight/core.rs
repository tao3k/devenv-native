//! Service-core state for the runtime-owned Wendao Flight server.

use std::sync::Arc;

use crate::transport::RerankScoreWeights;

use super::cache::FlightRoutePayloadCache;
use crate::transport::server::{
    AstSearchFlightRouteProvider, AttachmentSearchFlightRouteProvider,
    AutocompleteFlightRouteProvider, CodeAstAnalysisFlightRouteProvider,
    DefinitionFlightRouteProvider, DocumentExtractFlightRouteProvider,
    GraphNeighborsFlightRouteProvider, MarkdownAnalysisFlightRouteProvider,
    RefineDocFlightRouteProvider, RepoDocCoverageFlightRouteProvider, RepoIndexFlightRouteProvider,
    RepoIndexStatusFlightRouteProvider, RepoOverviewFlightRouteProvider,
    RepoProjectedPageIndexTreeFlightRouteProvider,
    RepoProjectedRetrievalContextFlightRouteProvider, RepoSearchFlightRouteProvider,
    RepoSyncFlightRouteProvider, RerankFlightRouteHandler, SearchFlightRouteProvider,
    SqlFlightRouteProvider, Topology3dFlightRouteProvider, VfsContentFlightRouteProvider,
    VfsResolveFlightRouteProvider, VfsScanFlightRouteProvider, WendaoFlightRouteProviders,
};

/// Runtime-owned minimal Wendao Flight service surface for the stable query and
/// rerank routes.
#[derive(Debug, Clone)]
pub struct WendaoFlightService {
    pub(crate) expected_schema_version: String,
    pub(super) repo_search_provider: Arc<dyn RepoSearchFlightRouteProvider>,
    pub(super) search_provider: Option<Arc<dyn SearchFlightRouteProvider>>,
    pub(super) attachment_search_provider: Option<Arc<dyn AttachmentSearchFlightRouteProvider>>,
    pub(super) ast_search_provider: Option<Arc<dyn AstSearchFlightRouteProvider>>,
    pub(super) definition_provider: Option<Arc<dyn DefinitionFlightRouteProvider>>,
    pub(super) autocomplete_provider: Option<Arc<dyn AutocompleteFlightRouteProvider>>,
    pub(super) vfs_content_provider: Option<Arc<dyn VfsContentFlightRouteProvider>>,
    pub(super) vfs_scan_provider: Option<Arc<dyn VfsScanFlightRouteProvider>>,
    pub(super) vfs_resolve_provider: Option<Arc<dyn VfsResolveFlightRouteProvider>>,
    pub(super) graph_neighbors_provider: Option<Arc<dyn GraphNeighborsFlightRouteProvider>>,
    pub(super) topology_3d_provider: Option<Arc<dyn Topology3dFlightRouteProvider>>,
    pub(super) markdown_analysis_provider: Option<Arc<dyn MarkdownAnalysisFlightRouteProvider>>,
    pub(super) code_ast_analysis_provider: Option<Arc<dyn CodeAstAnalysisFlightRouteProvider>>,
    pub(super) repo_overview_provider: Option<Arc<dyn RepoOverviewFlightRouteProvider>>,
    pub(super) repo_index_provider: Option<Arc<dyn RepoIndexFlightRouteProvider>>,
    pub(super) repo_index_status_provider: Option<Arc<dyn RepoIndexStatusFlightRouteProvider>>,
    pub(super) repo_sync_provider: Option<Arc<dyn RepoSyncFlightRouteProvider>>,
    pub(super) repo_doc_coverage_provider: Option<Arc<dyn RepoDocCoverageFlightRouteProvider>>,
    pub(super) repo_projected_page_index_tree_provider:
        Option<Arc<dyn RepoProjectedPageIndexTreeFlightRouteProvider>>,
    pub(super) repo_projected_retrieval_context_provider:
        Option<Arc<dyn RepoProjectedRetrievalContextFlightRouteProvider>>,
    pub(super) refine_doc_provider: Option<Arc<dyn RefineDocFlightRouteProvider>>,
    pub(super) document_extract_provider: Option<Arc<dyn DocumentExtractFlightRouteProvider>>,
    pub(super) sql_provider: Option<Arc<dyn SqlFlightRouteProvider>>,
    pub(super) rerank_handler: RerankFlightRouteHandler,
    pub(super) route_payload_cache: Arc<FlightRoutePayloadCache>,
}

impl WendaoFlightService {
    pub(super) fn build(
        expected_schema_version: String,
        route_providers: WendaoFlightRouteProviders,
        rerank_weights: RerankScoreWeights,
        rerank_dimension: usize,
    ) -> Result<Self, String> {
        let WendaoFlightRouteProviders {
            repo_search: repo_search_provider,
            search: search_provider,
            attachment_search: attachment_search_provider,
            ast_search: ast_search_provider,
            definition: definition_provider,
            autocomplete: autocomplete_provider,
            markdown_analysis: markdown_analysis_provider,
            code_ast_analysis: code_ast_analysis_provider,
            repo_overview: repo_overview_provider,
            repo_index: repo_index_provider,
            repo_index_status: repo_index_status_provider,
            repo_sync: repo_sync_provider,
            repo_doc_coverage: repo_doc_coverage_provider,
            repo_projected_page_index_tree: repo_projected_page_index_tree_provider,
            repo_projected_retrieval_context: repo_projected_retrieval_context_provider,
            refine_doc: refine_doc_provider,
            document_extract: document_extract_provider,
            vfs_content: vfs_content_provider,
            vfs_scan: vfs_scan_provider,
            vfs_resolve: vfs_resolve_provider,
            graph_neighbors: graph_neighbors_provider,
            topology_3d: topology_3d_provider,
            sql: sql_provider,
        } = route_providers;
        Ok(Self {
            expected_schema_version,
            repo_search_provider,
            search_provider,
            attachment_search_provider,
            ast_search_provider,
            definition_provider,
            autocomplete_provider,
            vfs_content_provider,
            vfs_scan_provider,
            vfs_resolve_provider,
            graph_neighbors_provider,
            topology_3d_provider,
            markdown_analysis_provider,
            code_ast_analysis_provider,
            repo_overview_provider,
            repo_index_provider,
            repo_index_status_provider,
            repo_sync_provider,
            repo_doc_coverage_provider,
            repo_projected_page_index_tree_provider,
            repo_projected_retrieval_context_provider,
            refine_doc_provider,
            document_extract_provider,
            sql_provider,
            rerank_handler: RerankFlightRouteHandler::new_with_weights(
                rerank_dimension,
                rerank_weights,
            )?,
            route_payload_cache: Arc::new(FlightRoutePayloadCache::default()),
        })
    }
}
