//! Request metadata validators grouped by Flight route family.

mod analysis;
mod document_extract;
mod ontology;
mod repo;
mod routing;
mod search;

pub(crate) use analysis::{
    validate_code_ast_analysis_request_metadata, validate_graph_neighbors_request_metadata,
    validate_markdown_analysis_request_metadata, validate_sql_request_metadata,
    validate_vfs_content_request_metadata, validate_vfs_resolve_request_metadata,
};
pub(crate) use document_extract::{
    validate_document_extract_request_metadata, validate_document_extract_status_request_metadata,
};
pub(crate) use ontology::validate_dataset_ontology_materialize_request_metadata;
pub(crate) use repo::{
    validate_refine_doc_request_metadata, validate_repo_doc_coverage_request_metadata,
    validate_repo_index_request_metadata, validate_repo_index_status_request_metadata,
    validate_repo_overview_request_metadata,
    validate_repo_projected_page_index_tree_request_metadata,
    validate_repo_projected_retrieval_context_request_metadata,
    validate_repo_sync_request_metadata,
};
pub(crate) use routing::{
    descriptor_route, is_search_family_route, join_sorted_set, ticket_route,
    validate_rerank_dimension_header, validate_rerank_min_final_score_header,
    validate_rerank_top_k_header, validate_schema_version,
};
pub(crate) use search::{
    validate_attachment_search_request_metadata, validate_autocomplete_request_metadata,
    validate_definition_request_metadata, validate_repo_search_request_metadata,
    validate_search_request_metadata,
};
