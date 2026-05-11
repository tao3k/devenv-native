use crate::transport::{
    ANALYSIS_REFINE_DOC_ROUTE, ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_INDEX_ROUTE,
    ANALYSIS_REPO_INDEX_STATUS_ROUTE, ANALYSIS_REPO_OVERVIEW_ROUTE,
    ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    WENDAO_REFINE_DOC_ENTITY_ID_HEADER, WENDAO_REFINE_DOC_REPO_HEADER,
    WENDAO_REFINE_DOC_USER_HINTS_HEADER, WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
    WENDAO_REPO_DOC_COVERAGE_REPO_HEADER, WENDAO_REPO_INDEX_REFRESH_HEADER,
    WENDAO_REPO_INDEX_REPO_HEADER, WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
    WENDAO_REPO_INDEX_STATUS_REPO_HEADER, WENDAO_REPO_OVERVIEW_REPO_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
    WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
};

#[test]
fn repo_doc_coverage_route_constant_is_stable() {
    assert_eq!(
        ANALYSIS_REPO_DOC_COVERAGE_ROUTE,
        "/analysis/repo-doc-coverage"
    );
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_REPO_HEADER,
        "x-wendao-repo-doc-coverage-repo"
    );
    assert_eq!(
        WENDAO_REPO_DOC_COVERAGE_MODULE_HEADER,
        "x-wendao-repo-doc-coverage-module"
    );
}

#[test]
fn repo_overview_route_constant_and_header_are_stable() {
    assert_eq!(ANALYSIS_REPO_OVERVIEW_ROUTE, "/analysis/repo-overview");
    assert_eq!(
        WENDAO_REPO_OVERVIEW_REPO_HEADER,
        "x-wendao-repo-overview-repo"
    );
}

#[test]
fn repo_index_status_route_constant_and_header_are_stable() {
    assert_eq!(
        ANALYSIS_REPO_INDEX_STATUS_ROUTE,
        "/analysis/repo-index-status"
    );
    assert_eq!(
        WENDAO_REPO_INDEX_STATUS_REPO_HEADER,
        "x-wendao-repo-index-status-repo"
    );
}

#[test]
fn repo_index_route_constants_and_headers_are_stable() {
    assert_eq!(ANALYSIS_REPO_INDEX_ROUTE, "/analysis/repo-index");
    assert_eq!(WENDAO_REPO_INDEX_REPO_HEADER, "x-wendao-repo-index-repo");
    assert_eq!(
        WENDAO_REPO_INDEX_REFRESH_HEADER,
        "x-wendao-repo-index-refresh"
    );
    assert_eq!(
        WENDAO_REPO_INDEX_REQUEST_ID_HEADER,
        "x-wendao-repo-index-request-id"
    );
}

#[test]
fn repo_projected_page_index_tree_route_constant_is_stable() {
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE,
        "/analysis/repo-projected-page-index-tree"
    );
}

#[test]
fn repo_projected_retrieval_context_route_constants_are_stable() {
    assert_eq!(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
        "/analysis/repo-projected-retrieval-context"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER,
        "x-wendao-repo-projected-retrieval-context-repo"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-page-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER,
        "x-wendao-repo-projected-retrieval-context-node-id"
    );
    assert_eq!(
        WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER,
        "x-wendao-repo-projected-retrieval-context-related-limit"
    );
}

#[test]
fn refine_doc_route_constant_and_headers_are_stable() {
    assert_eq!(ANALYSIS_REFINE_DOC_ROUTE, "/analysis/refine-doc");
    assert_eq!(WENDAO_REFINE_DOC_REPO_HEADER, "x-wendao-refine-doc-repo");
    assert_eq!(
        WENDAO_REFINE_DOC_ENTITY_ID_HEADER,
        "x-wendao-refine-doc-entity-id"
    );
    assert_eq!(
        WENDAO_REFINE_DOC_USER_HINTS_HEADER,
        "x-wendao-refine-doc-user-hints-b64"
    );
}
