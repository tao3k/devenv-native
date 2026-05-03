//! Server-side route provider traits and stream aliases for Wendao Flight.

use std::pin::Pin;
use std::sync::Arc;

use arrow_array::{
    Float64Array as LanceFloat64Array, Int32Array as LanceInt32Array, RecordBatch,
    RecordBatch as LanceRecordBatch, StringArray as LanceStringArray,
};
use arrow_flight::{ActionType, FlightData, FlightInfo, HandshakeResponse, PutResult};
use arrow_schema::{DataType as LanceDataType, Field as LanceField, Schema as LanceSchema};
use async_trait::async_trait;
use futures::Stream;
use tonic::Status;

use crate::transport::query_contract::{
    DocumentExtractFlightRequest, RERANK_RESPONSE_DOC_ID_COLUMN,
    RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN, RerankScoreWeights,
    score_rerank_request_batch_with_weights,
};

type EngineRecordBatch = LanceRecordBatch;

pub(crate) type FlightDataStream = Pin<Box<dyn Stream<Item = Result<FlightData, Status>> + Send>>;
pub(crate) type HandshakeStream =
    Pin<Box<dyn Stream<Item = Result<HandshakeResponse, Status>> + Send>>;
pub(crate) type PutResultStream = Pin<Box<dyn Stream<Item = Result<PutResult, Status>> + Send>>;
pub(crate) type ActionResultStream =
    Pin<Box<dyn Stream<Item = Result<arrow_flight::Result, Status>> + Send>>;
pub(crate) type FlightInfoStream = Pin<Box<dyn Stream<Item = Result<FlightInfo, Status>> + Send>>;
pub(crate) type ActionTypeStream = Pin<Box<dyn Stream<Item = Result<ActionType, Status>> + Send>>;

/// Transport-owned repo-search request decoded from Arrow Flight metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSearchFlightRequest {
    /// Repository identifier scoped by the caller.
    pub repo_id: String,
    /// Stable query text sent through the Flight route.
    pub query_text: String,
    /// Maximum number of rows requested from the provider.
    pub limit: usize,
    /// Optional language filters.
    pub language_filters: std::collections::HashSet<String>,
    /// Optional path-prefix filters.
    pub path_prefixes: std::collections::HashSet<String>,
    /// Optional title filters.
    pub title_filters: std::collections::HashSet<String>,
    /// Optional tag filters.
    pub tag_filters: std::collections::HashSet<String>,
    /// Optional filename filters.
    pub filename_filters: std::collections::HashSet<String>,
}

/// Transport-owned route-provider bundle used to build one Flight service.
#[derive(Debug, Clone)]
pub struct WendaoFlightRouteProviders {
    /// Mandatory repo-search provider.
    pub repo_search: Arc<dyn RepoSearchFlightRouteProvider>,
    /// Optional generic search-family provider.
    pub search: Option<Arc<dyn SearchFlightRouteProvider>>,
    /// Optional attachment-search provider.
    pub attachment_search: Option<Arc<dyn AttachmentSearchFlightRouteProvider>>,
    /// Optional AST-search provider.
    pub ast_search: Option<Arc<dyn AstSearchFlightRouteProvider>>,
    /// Optional definition provider.
    pub definition: Option<Arc<dyn DefinitionFlightRouteProvider>>,
    /// Optional autocomplete provider.
    pub autocomplete: Option<Arc<dyn AutocompleteFlightRouteProvider>>,
    /// Optional markdown-analysis provider.
    pub markdown_analysis: Option<Arc<dyn MarkdownAnalysisFlightRouteProvider>>,
    /// Optional code-AST-analysis provider.
    pub code_ast_analysis: Option<Arc<dyn CodeAstAnalysisFlightRouteProvider>>,
    /// Optional repo-overview analysis provider.
    pub repo_overview: Option<Arc<dyn RepoOverviewFlightRouteProvider>>,
    /// Optional repo-index analysis provider.
    pub repo_index: Option<Arc<dyn RepoIndexFlightRouteProvider>>,
    /// Optional repo-index-status analysis provider.
    pub repo_index_status: Option<Arc<dyn RepoIndexStatusFlightRouteProvider>>,
    /// Optional repo-sync analysis provider.
    pub repo_sync: Option<Arc<dyn RepoSyncFlightRouteProvider>>,
    /// Optional repo-doc-coverage analysis provider.
    pub repo_doc_coverage: Option<Arc<dyn RepoDocCoverageFlightRouteProvider>>,
    /// Optional projected page-index tree analysis provider.
    pub repo_projected_page_index_tree:
        Option<Arc<dyn RepoProjectedPageIndexTreeFlightRouteProvider>>,
    /// Optional refine-doc analysis provider.
    pub refine_doc: Option<Arc<dyn RefineDocFlightRouteProvider>>,
    /// Optional VFS-content provider.
    pub vfs_content: Option<Arc<dyn VfsContentFlightRouteProvider>>,
    /// Optional VFS-scan provider.
    pub vfs_scan: Option<Arc<dyn VfsScanFlightRouteProvider>>,
    /// Optional VFS-resolve provider.
    pub vfs_resolve: Option<Arc<dyn VfsResolveFlightRouteProvider>>,
    /// Optional graph-neighbors provider.
    pub graph_neighbors: Option<Arc<dyn GraphNeighborsFlightRouteProvider>>,
    /// Optional topology-3d provider.
    pub topology_3d: Option<Arc<dyn Topology3dFlightRouteProvider>>,
    /// Optional document extraction provider.
    pub document_extract: Option<Arc<dyn DocumentExtractFlightRouteProvider>>,
    /// Optional SQL provider.
    pub sql: Option<Arc<dyn SqlFlightRouteProvider>>,
}

impl WendaoFlightRouteProviders {
    /// Create one route-provider bundle with the mandatory repo-search provider.
    #[must_use]
    pub fn new(repo_search_provider: Arc<dyn RepoSearchFlightRouteProvider>) -> Self {
        Self {
            repo_search: repo_search_provider,
            search: None,
            attachment_search: None,
            ast_search: None,
            definition: None,
            autocomplete: None,
            markdown_analysis: None,
            code_ast_analysis: None,
            repo_overview: None,
            repo_index: None,
            repo_index_status: None,
            repo_sync: None,
            repo_doc_coverage: None,
            repo_projected_page_index_tree: None,
            refine_doc: None,
            vfs_content: None,
            vfs_scan: None,
            vfs_resolve: None,
            graph_neighbors: None,
            topology_3d: None,
            document_extract: None,
            sql: None,
        }
    }
}

/// Transport-owned generic search-family Flight payload.
#[derive(Debug, Clone)]
pub struct SearchFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl SearchFlightRouteResponse {
    /// Create one search-family Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned definition-resolution Flight payload.
#[derive(Debug, Clone)]
pub struct DefinitionFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl DefinitionFlightRouteResponse {
    /// Create one definition-resolution Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned autocomplete Flight payload.
#[derive(Debug, Clone)]
pub struct AutocompleteFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl AutocompleteFlightRouteResponse {
    /// Create one autocomplete Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned SQL Flight payload.
#[derive(Debug, Clone)]
pub struct SqlFlightRouteResponse {
    /// Arrow batches returned by the provider.
    pub batches: Vec<EngineRecordBatch>,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl SqlFlightRouteResponse {
    /// Create one SQL Flight payload without application metadata.
    #[must_use]
    pub fn new(batches: Vec<EngineRecordBatch>) -> Self {
        Self {
            batches,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned VFS navigation-resolution Flight payload.
#[derive(Debug, Clone)]
pub struct VfsResolveFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl VfsResolveFlightRouteResponse {
    /// Create one VFS navigation-resolution Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned VFS content-read Flight payload.
#[derive(Debug, Clone)]
pub struct VfsContentFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl VfsContentFlightRouteResponse {
    /// Create one VFS content-read Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned VFS scan Flight payload.
#[derive(Debug, Clone)]
pub struct VfsScanFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl VfsScanFlightRouteResponse {
    /// Create one VFS scan Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned graph-neighbors Flight payload.
#[derive(Debug, Clone)]
pub struct GraphNeighborsFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl GraphNeighborsFlightRouteResponse {
    /// Create one graph-neighbors Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned topology-3d Flight payload.
#[derive(Debug, Clone)]
pub struct Topology3dFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl Topology3dFlightRouteResponse {
    /// Create one topology-3d Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned generic analysis-family Flight payload.
#[derive(Debug, Clone)]
pub struct AnalysisFlightRouteResponse {
    /// Arrow batch returned by the provider.
    pub batch: LanceRecordBatch,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl AnalysisFlightRouteResponse {
    /// Create one analysis-family Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: LanceRecordBatch) -> Self {
        Self {
            batch,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned document extraction Flight payload.
#[derive(Debug, Clone)]
pub struct DocumentExtractFlightRouteResponse {
    /// Arrow batches returned by the provider.
    pub batches: Vec<RecordBatch>,
    /// Optional application metadata returned through `FlightInfo.app_metadata`.
    pub app_metadata: Vec<u8>,
}

impl DocumentExtractFlightRouteResponse {
    /// Create one document extraction Flight payload without application metadata.
    #[must_use]
    pub fn new(batch: RecordBatch) -> Self {
        Self {
            batches: vec![batch],
            app_metadata: Vec::new(),
        }
    }

    /// Create a document extraction Flight payload from already materialized
    /// Arrow batches.
    #[must_use]
    pub fn from_batches(batches: Vec<RecordBatch>) -> Self {
        Self {
            batches,
            app_metadata: Vec::new(),
        }
    }

    /// Attach application metadata that should flow through `FlightInfo`.
    #[must_use]
    pub fn with_app_metadata(mut self, app_metadata: impl Into<Vec<u8>>) -> Self {
        self.app_metadata = app_metadata.into();
        self
    }
}

/// Transport-owned provider contract for stable repo-search Flight reads.
#[async_trait]
pub trait RepoSearchFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve a stable repo-search response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo-search payload cannot be
    /// materialized for the current transport host.
    async fn repo_search_batch(
        &self,
        request: &RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String>;
}

/// Transport-owned provider contract for stable generic search-family Flight
/// reads.
#[async_trait]
pub trait SearchFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable search-family response batch for the requested route.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested search-family payload cannot be
    /// materialized for the current transport host.
    async fn search_batch(
        &self,
        route: &str,
        query_text: &str,
        limit: usize,
        intent: Option<&str>,
        repo_hint: Option<&str>,
    ) -> Result<SearchFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable definition-resolution Flight
/// reads.
#[async_trait]
pub trait DefinitionFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable definition-resolution response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested definition payload
    /// cannot be materialized for the current transport host.
    async fn definition_batch(
        &self,
        query_text: &str,
        source_path: Option<&str>,
        source_line: Option<usize>,
    ) -> Result<DefinitionFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable autocomplete Flight reads.
#[async_trait]
pub trait AutocompleteFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable autocomplete response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested autocomplete payload
    /// cannot be materialized for the current transport host.
    async fn autocomplete_batch(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<AutocompleteFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable read-only SQL Flight reads.
#[async_trait]
pub trait SqlFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable SQL response batch sequence.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested SQL payload cannot be materialized
    /// for the current transport host.
    async fn sql_query_batches(&self, query_text: &str) -> Result<SqlFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable VFS navigation-resolution Flight
/// reads.
#[async_trait]
pub trait VfsResolveFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable VFS navigation target response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested VFS path cannot be
    /// materialized for the current transport host.
    async fn resolve_vfs_navigation_batch(
        &self,
        path: &str,
    ) -> Result<VfsResolveFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable VFS content Flight reads.
#[async_trait]
pub trait VfsContentFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable VFS content batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested VFS content payload
    /// cannot be materialized for the current transport host.
    async fn read_vfs_content_batch(
        &self,
        path: &str,
    ) -> Result<VfsContentFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable VFS scan Flight reads.
#[async_trait]
pub trait VfsScanFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable VFS scan response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested VFS scan payload
    /// cannot be materialized for the current transport host.
    async fn scan_vfs_batch(&self) -> Result<VfsScanFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable graph-neighbors Flight reads.
#[async_trait]
pub trait GraphNeighborsFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable graph-neighbors response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested graph-neighbors payload
    /// cannot be materialized for the current transport host.
    async fn graph_neighbors_batch(
        &self,
        node_id: &str,
        direction: &str,
        hops: usize,
        limit: usize,
    ) -> Result<GraphNeighborsFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable topology-3d Flight reads.
#[async_trait]
pub trait Topology3dFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable topology-3d response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested topology payload
    /// cannot be materialized for the current transport host.
    async fn topology_3d_batch(&self) -> Result<Topology3dFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable attachment-search Flight reads.
#[async_trait]
pub trait AttachmentSearchFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable attachment-search response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested attachment-search payload cannot be
    /// materialized for the current transport host.
    async fn attachment_search_batch(
        &self,
        query_text: &str,
        limit: usize,
        ext_filters: &std::collections::HashSet<String>,
        kind_filters: &std::collections::HashSet<String>,
        case_sensitive: bool,
    ) -> Result<SearchFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable AST-search Flight reads.
#[async_trait]
pub trait AstSearchFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable AST-search response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested AST-search payload cannot be
    /// materialized for the current transport host.
    async fn ast_search_batch(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<SearchFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable markdown analysis Flight reads.
#[async_trait]
pub trait MarkdownAnalysisFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable markdown analysis response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested markdown analysis payload cannot be
    /// materialized for the current transport host.
    async fn markdown_analysis_batch(
        &self,
        path: &str,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable document extraction Flight reads.
#[async_trait]
pub trait DocumentExtractFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable document extraction response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested document extraction payload cannot be
    /// materialized for the current transport host.
    async fn document_extract_batch(
        &self,
        source_path: &str,
        output_dir: &str,
        force: bool,
        error_row: bool,
    ) -> Result<DocumentExtractFlightRouteResponse, String>;

    /// Resolve a document extraction request with the latest metadata shape.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested document extraction payload cannot
    /// be materialized for the current transport host.
    async fn document_extract_batch_for_request(
        &self,
        request: &DocumentExtractFlightRequest,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        self.document_extract_batch(
            request.source_path.as_str(),
            request.output_dir.as_str(),
            request.force,
            request.error_row,
        )
        .await
    }

    /// Resolve one Rust-owned document extraction job status batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the job id is unknown or cannot be read.
    async fn document_extract_status_batch(
        &self,
        job_id: &str,
    ) -> Result<DocumentExtractFlightRouteResponse, String> {
        Err(format!(
            "document extract status route is not configured for job `{job_id}`"
        ))
    }
}

/// Transport-owned provider contract for stable code-AST analysis Flight reads.
#[async_trait]
pub trait CodeAstAnalysisFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable code-AST analysis response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested code-AST analysis payload cannot be
    /// materialized for the current transport host.
    async fn code_ast_analysis_batch(
        &self,
        path: &str,
        repo_id: &str,
        line_hint: Option<usize>,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable repo doc-coverage Flight reads.
#[async_trait]
pub trait RepoOverviewFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable repo overview response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo overview payload cannot be
    /// materialized for the current transport host.
    async fn repo_overview_batch(
        &self,
        repo_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable repo index Flight commands.
#[async_trait]
pub trait RepoIndexFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable repo index response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo index payload cannot be
    /// materialized for the current transport host.
    async fn repo_index_batch(
        &self,
        repo_id: Option<&str>,
        refresh: bool,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable repo index-status Flight reads.
#[async_trait]
pub trait RepoIndexStatusFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable repo index-status response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo index-status payload cannot be
    /// materialized for the current transport host.
    async fn repo_index_status_batch(
        &self,
        repo_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable repo sync Flight reads.
#[async_trait]
pub trait RepoSyncFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable repo sync response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo sync payload cannot be
    /// materialized for the current transport host.
    async fn repo_sync_batch(
        &self,
        repo_id: &str,
        mode: &str,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable repo doc-coverage Flight reads.
#[async_trait]
pub trait RepoDocCoverageFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable repo doc-coverage response batch.
    ///
    /// # Errors
    ///
    /// Returns an error when the requested repo doc-coverage payload cannot be
    /// materialized for the current transport host.
    async fn repo_doc_coverage_batch(
        &self,
        repo_id: &str,
        module_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String>;
}

/// Transport-owned provider contract for stable projected page-index tree Flight
/// reads.
#[async_trait]
pub trait RepoProjectedPageIndexTreeFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable projected page-index tree response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested projected page-index
    /// tree payload cannot be materialized for the current transport host.
    async fn repo_projected_page_index_tree_batch(
        &self,
        repo_id: &str,
        page_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, Status>;
}

/// Transport-owned provider contract for stable refine-doc Flight reads.
#[async_trait]
pub trait RefineDocFlightRouteProvider: std::fmt::Debug + Send + Sync {
    /// Resolve one stable refine-doc response batch.
    ///
    /// # Errors
    ///
    /// Returns a typed Flight status when the requested refine-doc payload
    /// cannot be materialized for the current transport host.
    async fn refine_doc_batch(
        &self,
        repo_id: &str,
        entity_id: &str,
        user_hints: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, Status>;
}

#[derive(Debug, Clone)]
pub(crate) struct StaticRepoSearchFlightRouteProvider {
    pub(super) batch: LanceRecordBatch,
}

#[async_trait]
impl RepoSearchFlightRouteProvider for StaticRepoSearchFlightRouteProvider {
    async fn repo_search_batch(
        &self,
        _request: &RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String> {
        Ok(self.batch.clone())
    }
}

/// Transport-owned server-side handler for the stable rerank Flight exchange route.
#[derive(Debug, Clone, Copy)]
pub struct RerankFlightRouteHandler {
    expected_dimension: usize,
    weights: RerankScoreWeights,
}

impl RerankFlightRouteHandler {
    /// Create one rerank Flight route handler.
    ///
    /// # Errors
    ///
    /// Returns an error when the expected embedding dimension is zero.
    pub fn new(expected_dimension: usize) -> Result<Self, String> {
        Self::new_with_weights(expected_dimension, RerankScoreWeights::default())
    }

    /// Create one rerank Flight route handler with explicit transport-owned
    /// score weights.
    ///
    /// # Errors
    ///
    /// Returns an error when the expected embedding dimension is zero or when
    /// the transport weights are invalid.
    pub fn new_with_weights(
        expected_dimension: usize,
        weights: RerankScoreWeights,
    ) -> Result<Self, String> {
        if expected_dimension == 0 {
            return Err("rerank route expected_dimension must be greater than zero".to_string());
        }
        Ok(Self {
            expected_dimension,
            weights: RerankScoreWeights::new(weights.vector_weight, weights.semantic_weight)?,
        })
    }

    /// Build one stable rerank response batch from decoded request batches.
    ///
    /// # Errors
    ///
    /// Returns an error when any request batch fails the shared rerank request
    /// contract, when the combined candidate list is empty, or when the
    /// response batch cannot be represented as the Lance-backed rerank output.
    pub fn handle_exchange_batches(
        &self,
        request_batches: &[RecordBatch],
        top_k: Option<usize>,
        min_final_score: Option<f64>,
    ) -> Result<LanceRecordBatch, String> {
        let mut scored_candidates = Vec::new();
        for batch in request_batches {
            scored_candidates.extend(score_rerank_request_batch_with_weights(
                batch,
                self.expected_dimension,
                self.weights,
            )?);
        }

        if scored_candidates.is_empty() {
            return Err("rerank request batches must contain at least one row".to_string());
        }

        if let Some(threshold) = min_final_score {
            scored_candidates.retain(|candidate| candidate.final_score >= threshold);
        }

        scored_candidates.sort_by(|left, right| {
            right
                .final_score
                .partial_cmp(&left.final_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.doc_id.cmp(&right.doc_id))
        });
        if let Some(limit) = top_k {
            scored_candidates.truncate(limit);
        }

        let doc_ids = scored_candidates
            .iter()
            .map(|candidate| candidate.doc_id.clone())
            .collect::<Vec<_>>();
        let vector_scores = scored_candidates
            .iter()
            .map(|candidate| candidate.vector_score)
            .collect::<Vec<_>>();
        let semantic_scores = scored_candidates
            .iter()
            .map(|candidate| candidate.semantic_score)
            .collect::<Vec<_>>();
        let final_scores = scored_candidates
            .iter()
            .map(|candidate| candidate.final_score)
            .collect::<Vec<_>>();
        let ranks = (1..=i32::try_from(scored_candidates.len())
            .map_err(|error| format!("failed to represent rerank response rank range: {error}"))?)
            .collect::<Vec<_>>();

        LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new(RERANK_RESPONSE_DOC_ID_COLUMN, LanceDataType::Utf8, false),
                LanceField::new(
                    RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
                    LanceDataType::Float64,
                    false,
                ),
                LanceField::new(
                    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN,
                    LanceDataType::Float64,
                    false,
                ),
                LanceField::new(
                    RERANK_RESPONSE_FINAL_SCORE_COLUMN,
                    LanceDataType::Float64,
                    false,
                ),
                LanceField::new(RERANK_RESPONSE_RANK_COLUMN, LanceDataType::Int32, false),
            ])),
            vec![
                Arc::new(LanceStringArray::from(doc_ids)),
                Arc::new(LanceFloat64Array::from(vector_scores)),
                Arc::new(LanceFloat64Array::from(semantic_scores)),
                Arc::new(LanceFloat64Array::from(final_scores)),
                Arc::new(LanceInt32Array::from(ranks)),
            ],
        )
        .map_err(|error| format!("failed to build rerank response batch: {error}"))
    }
}
