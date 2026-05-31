//! Transport-owned server boundary for Arrow Flight route services.

mod flight;
mod ontology;
mod request_metadata;
mod sample_host;
mod types;

pub use flight::WendaoFlightService;
pub use ontology::{
    DatasetOntologyMaterializeFlightRouteProvider, DatasetOntologyMaterializeFlightRouteResponse,
    OntologyCandidateInspectionFlightRouteProvider, OntologyCandidateInspectionFlightRouteResponse,
};
pub use sample_host::run_wendao_flight_server_from_args;
pub use types::{
    AnalysisFlightRouteResponse, AstSearchFlightRouteProvider, AttachmentSearchFlightRouteProvider,
    AttachmentSearchFlightRouteRequest, AutocompleteFlightRouteProvider,
    AutocompleteFlightRouteResponse, CodeAstAnalysisFlightRouteProvider,
    DefinitionFlightRouteProvider, DefinitionFlightRouteResponse,
    DocumentExtractFlightRouteProvider, DocumentExtractFlightRouteResponse,
    GraphNeighborsFlightRouteProvider, GraphNeighborsFlightRouteResponse,
    MarkdownAnalysisFlightRouteProvider, RefineDocFlightRouteProvider,
    RepoDocCoverageFlightRouteProvider, RepoIndexFlightRouteProvider,
    RepoIndexStatusFlightRouteProvider, RepoOverviewFlightRouteProvider,
    RepoProjectedPageIndexTreeFlightRouteProvider,
    RepoProjectedRetrievalContextFlightRouteProvider, RepoSearchFlightRequest,
    RepoSearchFlightRouteProvider, RepoSyncFlightRouteProvider, RerankFlightRouteHandler,
    SearchFlightRouteProvider, SearchFlightRouteRequest, SearchFlightRouteResponse,
    SemanticScopeFlightRouteProvider, SqlFlightRouteProvider, SqlFlightRouteResponse,
    Topology3dFlightRouteProvider, Topology3dFlightRouteResponse, VfsContentFlightRouteProvider,
    VfsContentFlightRouteResponse, VfsResolveFlightRouteProvider, VfsResolveFlightRouteResponse,
    VfsScanFlightRouteProvider, VfsScanFlightRouteResponse, WendaoFlightRouteProviders,
};

pub(crate) use request_metadata::{
    descriptor_route, is_search_family_route, join_sorted_set, ticket_route,
    validate_attachment_search_request_metadata, validate_autocomplete_request_metadata,
    validate_code_ast_analysis_request_metadata,
    validate_dataset_ontology_materialize_request_metadata, validate_definition_request_metadata,
    validate_document_extract_request_metadata, validate_document_extract_status_request_metadata,
    validate_graph_neighbors_request_metadata, validate_markdown_analysis_request_metadata,
    validate_ontology_candidate_inspection_request_metadata, validate_refine_doc_request_metadata,
    validate_repo_doc_coverage_request_metadata, validate_repo_index_request_metadata,
    validate_repo_index_status_request_metadata, validate_repo_overview_request_metadata,
    validate_repo_projected_page_index_tree_request_metadata,
    validate_repo_projected_retrieval_context_request_metadata,
    validate_repo_search_request_metadata, validate_repo_sync_request_metadata,
    validate_rerank_dimension_header, validate_rerank_min_final_score_header,
    validate_rerank_top_k_header, validate_schema_version, validate_search_request_metadata,
    validate_semantic_scope_request_metadata, validate_sql_request_metadata,
    validate_vfs_content_request_metadata, validate_vfs_resolve_request_metadata,
};

pub(crate) use types::{
    ActionResultStream, ActionTypeStream, FlightDataStream, FlightInfoStream, HandshakeStream,
    PutResultStream, StaticRepoSearchFlightRouteProvider,
};
