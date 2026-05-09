//! Repository-analysis route contracts for Wendao Flight metadata.

mod doc_coverage;
mod index;
mod index_status;
mod overview;
mod projected_page_index_tree;
mod projected_retrieval_context;
mod refine_doc;
mod sync;

pub use doc_coverage::{
    ANALYSIS_REPO_DOC_COVERAGE_ROUTE, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, validate_repo_doc_coverage_request,
};
pub use index::{
    ANALYSIS_REPO_INDEX_ROUTE, WENDAO_REPO_INDEX_REFRESH_HEADER, WENDAO_REPO_INDEX_REPO_HEADER,
    WENDAO_REPO_INDEX_REQUEST_ID_HEADER, validate_repo_index_request,
};
pub use index_status::{
    ANALYSIS_REPO_INDEX_STATUS_ROUTE, WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
    validate_repo_index_status_request,
};
pub use overview::{
    ANALYSIS_REPO_OVERVIEW_ROUTE, WENDAO_REPO_OVERVIEW_REPO_HEADER, validate_repo_overview_request,
};
pub use projected_page_index_tree::{
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER,
    validate_repo_projected_page_index_tree_request,
};
pub use projected_retrieval_context::{
    ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
    validate_repo_projected_retrieval_context_request,
};
pub use refine_doc::{
    ANALYSIS_REFINE_DOC_ROUTE, WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, validate_refine_doc_request,
};
pub use sync::{
    ANALYSIS_REPO_SYNC_ROUTE, WENDAO_REPO_SYNC_MODE_HEADER, WENDAO_REPO_SYNC_REPO_HEADER,
    validate_repo_sync_request,
};
