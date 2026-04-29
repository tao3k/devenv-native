#[cfg(feature = "transport")]
mod client;
#[cfg(feature = "transport")]
mod contract;
#[cfg(feature = "transport")]
mod flight;
#[cfg(feature = "transport")]
mod host_settings;
#[cfg(feature = "transport")]
mod negotiation;
#[cfg(feature = "transport")]
mod plugin_arrow_exchange;
mod query_contract;
#[cfg(feature = "transport")]
mod server;

#[cfg(feature = "transport")]
pub use contract::{
    DEFAULT_FLIGHT_BASE_URL, DEFAULT_FLIGHT_MAX_IN_FLIGHT_REQUESTS, DEFAULT_FLIGHT_SCHEMA_VERSION,
    DEFAULT_FLIGHT_TIMEOUT_SECS, FLIGHT_SCHEMA_VERSION_METADATA_KEY, FLIGHT_TRACE_ID_METADATA_KEY,
    resolve_default_flight_base_url, resolve_flight_max_in_flight_requests, resolve_flight_timeout,
    validate_flight_max_in_flight_requests, validate_flight_schema_version,
    validate_flight_timeout_secs,
};
#[cfg(feature = "transport")]
pub use host_settings::{
    EffectiveRerankFlightHostSettings, ParsedRerankFlightHostOverrides,
    rerank_score_weights_from_env, resolve_effective_rerank_flight_host_settings,
    split_rerank_flight_host_overrides,
};
#[cfg(feature = "transport")]
pub use negotiation::{
    CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER, NegotiatedFlightTransportClient,
    NegotiatedTransportSelection, negotiate_flight_transport_client_from_bindings,
};
#[cfg(feature = "transport")]
pub use plugin_arrow_exchange::{
    NegotiatedPluginArrowScoreRows, PluginArrowCandidateProjection,
    PluginArrowRequestBatchBuildError, PluginArrowRequestRow, PluginArrowScoreRoundtripError,
    PluginArrowScoreRow, PluginArrowScoredCandidate, attach_plugin_arrow_request_metadata,
    build_plugin_arrow_request_batch, build_plugin_arrow_request_batch_from_embeddings,
    build_plugin_arrow_request_batch_from_embeddings_with_metadata, decode_plugin_arrow_score_rows,
    plugin_arrow_request_trace_id, project_plugin_arrow_scored_candidates,
    roundtrip_plugin_arrow_score_rows_with_binding, validate_plugin_arrow_response_batches,
};
#[cfg(all(feature = "transport", feature = "vector-store"))]
pub use plugin_arrow_exchange::{
    PluginArrowVectorStoreRequestBuildError, build_plugin_arrow_request_batch_from_vector_store,
    build_plugin_arrow_request_batch_from_vector_store_with_metadata,
    prepare_plugin_arrow_request_rows_from_vector_store,
};
pub use query_contract::RerankScoreWeights;
#[cfg(feature = "transport")]
pub use query_contract::validate_sql_query_request;
pub use query_contract::{
    ANALYSIS_CODE_AST_ROUTE, ANALYSIS_DOCUMENT_EXTRACT_ROUTE,
    ANALYSIS_DOCUMENT_EXTRACT_STATUS_ROUTE, ANALYSIS_MARKDOWN_ROUTE, ANALYSIS_REFINE_DOC_ROUTE,
    ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_ROUTE, ANALYSIS_REPO_INDEX_STATUS_ROUTE,
    ANALYSIS_REPO_OVERVIEW_ROUTE, ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
    ANALYSIS_REPO_SYNC_ROUTE, DocumentExtractFlightRequest, DocumentExtractMode,
    GRAPH_NEIGHBORS_DEFAULT_HOPS, GRAPH_NEIGHBORS_DEFAULT_LIMIT, GRAPH_NEIGHBORS_ROUTE,
    QUERY_SQL_ROUTE, REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DEFAULT_LIMIT,
    REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_HIERARCHY_COLUMN, REPO_SEARCH_LANGUAGE_COLUMN,
    REPO_SEARCH_MATCH_REASON_COLUMN, REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_COLUMN, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
    REPO_SEARCH_NAVIGATION_PATH_COLUMN, REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_ROUTE,
    REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TAGS_COLUMN, REPO_SEARCH_TITLE_COLUMN, RERANK_ROUTE,
    SEARCH_AST_ROUTE, SEARCH_ATTACHMENTS_ROUTE, SEARCH_AUTOCOMPLETE_ROUTE, SEARCH_DEFINITION_ROUTE,
    SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE, SEARCH_SYMBOLS_ROUTE,
    TOPOLOGY_3D_ROUTE, VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, VFS_SCAN_ROUTE,
    WENDAO_ANALYSIS_LINE_HEADER, WENDAO_ANALYSIS_PATH_HEADER, WENDAO_ANALYSIS_REPO_HEADER,
    WENDAO_ATTACHMENT_SEARCH_CASE_SENSITIVE_HEADER, WENDAO_ATTACHMENT_SEARCH_EXT_FILTERS_HEADER,
    WENDAO_ATTACHMENT_SEARCH_KIND_FILTERS_HEADER, WENDAO_AUTOCOMPLETE_LIMIT_HEADER,
    WENDAO_AUTOCOMPLETE_PREFIX_HEADER, WENDAO_DEFINITION_LINE_HEADER,
    WENDAO_DEFINITION_PATH_HEADER, WENDAO_DEFINITION_QUERY_HEADER,
    WENDAO_DOCUMENT_EXTRACT_ERROR_ROW_HEADER, WENDAO_DOCUMENT_EXTRACT_FORCE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_JOB_ID_HEADER, WENDAO_DOCUMENT_EXTRACT_MODE_HEADER,
    WENDAO_DOCUMENT_EXTRACT_OUTPUT_DIR_HEADER, WENDAO_DOCUMENT_EXTRACT_SOURCE_PATH_HEADER,
    WENDAO_DOCUMENT_EXTRACT_WAIT_MS_HEADER, WENDAO_GRAPH_DIRECTION_HEADER,
    WENDAO_GRAPH_HOPS_HEADER, WENDAO_GRAPH_LIMIT_HEADER, WENDAO_GRAPH_NODE_ID_HEADER,
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER, WENDAO_REPO_SEARCH_FILENAME_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_LANGUAGE_FILTERS_HEADER, WENDAO_REPO_SEARCH_LIMIT_HEADER,
    WENDAO_REPO_SEARCH_PATH_PREFIXES_HEADER, WENDAO_REPO_SEARCH_QUERY_HEADER,
    WENDAO_REPO_SEARCH_REPO_HEADER, WENDAO_REPO_SEARCH_TAG_FILTERS_HEADER,
    WENDAO_REPO_SEARCH_TITLE_FILTERS_HEADER, WENDAO_REPO_SYNC_MODE_HEADER,
    WENDAO_REPO_SYNC_REPO_HEADER, WENDAO_RERANK_DIMENSION_HEADER, WENDAO_SCHEMA_VERSION_HEADER,
    WENDAO_SEARCH_INTENT_HEADER, WENDAO_SEARCH_LIMIT_HEADER, WENDAO_SEARCH_QUERY_HEADER,
    WENDAO_SEARCH_REPO_HEADER, WENDAO_SQL_QUERY_HEADER, WENDAO_VFS_PATH_HEADER,
    flight_descriptor_path, normalize_flight_route, validate_attachment_search_request,
    validate_autocomplete_request, validate_code_ast_analysis_request, validate_definition_request,
    validate_document_extract_request, validate_graph_neighbors_request,
    validate_markdown_analysis_request, validate_refine_doc_request,
    validate_repo_doc_coverage_request, validate_repo_index_request,
    validate_repo_index_status_request, validate_repo_overview_request,
    validate_repo_projected_page_index_tree_request, validate_repo_search_request,
    validate_repo_sync_request, validate_vfs_content_request, validate_vfs_resolve_request,
};
#[cfg(feature = "transport")]
pub use query_contract::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    RERANK_RESPONSE_DOC_ID_COLUMN, RERANK_RESPONSE_FINAL_SCORE_COLUMN, RERANK_RESPONSE_RANK_COLUMN,
    RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN, RERANK_RESPONSE_VECTOR_SCORE_COLUMN,
};
#[cfg(feature = "transport")]
pub use query_contract::{
    RerankScoredCandidate, score_rerank_request_batch, score_rerank_request_batch_with_weights,
    validate_rerank_request_batch, validate_rerank_request_schema, validate_rerank_response_batch,
    validate_rerank_response_schema,
};
#[cfg(feature = "transport")]
pub use server::{
    AnalysisFlightRouteResponse, AstSearchFlightRouteProvider, AttachmentSearchFlightRouteProvider,
    AutocompleteFlightRouteProvider, AutocompleteFlightRouteResponse,
    CodeAstAnalysisFlightRouteProvider, DefinitionFlightRouteProvider,
    DefinitionFlightRouteResponse, DocumentExtractFlightRouteProvider,
    DocumentExtractFlightRouteResponse, GraphNeighborsFlightRouteProvider,
    GraphNeighborsFlightRouteResponse, MarkdownAnalysisFlightRouteProvider,
    RefineDocFlightRouteProvider, RepoDocCoverageFlightRouteProvider, RepoIndexFlightRouteProvider,
    RepoIndexStatusFlightRouteProvider, RepoOverviewFlightRouteProvider,
    RepoProjectedPageIndexTreeFlightRouteProvider, RepoSearchFlightRequest,
    RepoSearchFlightRouteProvider, RepoSyncFlightRouteProvider, RerankFlightRouteHandler,
    SearchFlightRouteProvider, SearchFlightRouteResponse, SqlFlightRouteProvider,
    SqlFlightRouteResponse, Topology3dFlightRouteProvider, Topology3dFlightRouteResponse,
    VfsContentFlightRouteProvider, VfsContentFlightRouteResponse, VfsResolveFlightRouteProvider,
    VfsResolveFlightRouteResponse, VfsScanFlightRouteProvider, VfsScanFlightRouteResponse,
    WendaoFlightRouteProviders, WendaoFlightService,
};

#[cfg(all(feature = "transport", test))]
pub(crate) use query_contract::WENDAO_RERANK_TOP_K_HEADER;
#[cfg(all(feature = "transport", test))]
pub(crate) use server::{
    is_search_family_route, validate_attachment_search_request_metadata,
    validate_autocomplete_request_metadata, validate_code_ast_analysis_request_metadata,
    validate_definition_request_metadata, validate_document_extract_request_metadata,
    validate_document_extract_status_request_metadata, validate_graph_neighbors_request_metadata,
    validate_markdown_analysis_request_metadata, validate_repo_doc_coverage_request_metadata,
    validate_repo_index_status_request_metadata, validate_repo_overview_request_metadata,
    validate_repo_search_request_metadata, validate_repo_sync_request_metadata,
    validate_rerank_top_k_header, validate_search_request_metadata, validate_sql_request_metadata,
    validate_vfs_content_request_metadata, validate_vfs_resolve_request_metadata,
};
