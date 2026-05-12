//! `gateway::openapi::paths::repo` owns Wendao openapi paths repo behavior.

/// Axum runtime path for the repo overview endpoint.
pub const API_REPO_OVERVIEW_AXUM_PATH: &str = "/api/repo/overview";
/// `OpenAPI` path for the repo overview endpoint.
pub const API_REPO_OVERVIEW_OPENAPI_PATH: &str = "/api/repo/overview";
/// Axum runtime path for the repo module-search endpoint.
pub const API_REPO_MODULE_SEARCH_AXUM_PATH: &str = "/api/repo/module-search";
/// `OpenAPI` path for the repo module-search endpoint.
pub const API_REPO_MODULE_SEARCH_OPENAPI_PATH: &str = "/api/repo/module-search";
/// Axum runtime path for the repo symbol-search endpoint.
pub const API_REPO_SYMBOL_SEARCH_AXUM_PATH: &str = "/api/repo/symbol-search";
/// `OpenAPI` path for the repo symbol-search endpoint.
pub const API_REPO_SYMBOL_SEARCH_OPENAPI_PATH: &str = "/api/repo/symbol-search";
/// Axum runtime path for the repo example-search endpoint.
pub const API_REPO_EXAMPLE_SEARCH_AXUM_PATH: &str = "/api/repo/example-search";
/// `OpenAPI` path for the repo example-search endpoint.
pub const API_REPO_EXAMPLE_SEARCH_OPENAPI_PATH: &str = "/api/repo/example-search";
/// Axum runtime path for the repo import-search endpoint.
pub const API_REPO_IMPORT_SEARCH_AXUM_PATH: &str = "/api/repo/import-search";
/// `OpenAPI` path for the repo import-search endpoint.
pub const API_REPO_IMPORT_SEARCH_OPENAPI_PATH: &str = "/api/repo/import-search";
/// Axum runtime path for the repo doc-coverage endpoint.
pub const API_REPO_DOC_COVERAGE_AXUM_PATH: &str = "/api/repo/doc-coverage";
/// `OpenAPI` path for the repo doc-coverage endpoint.
pub const API_REPO_DOC_COVERAGE_OPENAPI_PATH: &str = "/api/repo/doc-coverage";
/// Axum runtime path for the repo sync endpoint.
pub const API_REPO_SYNC_AXUM_PATH: &str = "/api/repo/sync";
/// `OpenAPI` path for the repo sync endpoint.
pub const API_REPO_SYNC_OPENAPI_PATH: &str = "/api/repo/sync";
/// Axum runtime path for the repo index enqueue endpoint.
pub const API_REPO_INDEX_AXUM_PATH: &str = "/api/repo/index";
/// `OpenAPI` path for the repo index enqueue endpoint.
pub const API_REPO_INDEX_OPENAPI_PATH: &str = "/api/repo/index";
/// Axum runtime path for the repo index status endpoint.
pub const API_REPO_INDEX_STATUS_AXUM_PATH: &str = "/api/repo/index/status";
/// `OpenAPI` path for the repo index status endpoint.
pub const API_REPO_INDEX_STATUS_OPENAPI_PATH: &str = "/api/repo/index/status";
/// Axum runtime path for the repo projected-pages endpoint.
pub const API_REPO_PROJECTED_PAGES_AXUM_PATH: &str = "/api/repo/projected-pages";
/// `OpenAPI` path for the repo projected-pages endpoint.
pub const API_REPO_PROJECTED_PAGES_OPENAPI_PATH: &str = "/api/repo/projected-pages";
/// Axum runtime path for the repo projected-gap-report endpoint.
pub const API_REPO_PROJECTED_GAP_REPORT_AXUM_PATH: &str = "/api/repo/projected-gap-report";
/// `OpenAPI` path for the repo projected-gap-report endpoint.
pub const API_REPO_PROJECTED_GAP_REPORT_OPENAPI_PATH: &str = "/api/repo/projected-gap-report";
/// Axum runtime path for the repo projected-page endpoint.
pub const API_REPO_PROJECTED_PAGE_AXUM_PATH: &str = "/api/repo/projected-page";
/// `OpenAPI` path for the repo projected-page endpoint.
pub const API_REPO_PROJECTED_PAGE_OPENAPI_PATH: &str = "/api/repo/projected-page";
/// Axum runtime path for the repo projected-page-index-node endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_NODE_AXUM_PATH: &str =
    "/api/repo/projected-page-index-node";
/// `OpenAPI` path for the repo projected-page-index-node endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_NODE_OPENAPI_PATH: &str =
    "/api/repo/projected-page-index-node";
/// Axum runtime path for the repo projected-retrieval-hit endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_HIT_AXUM_PATH: &str = "/api/repo/projected-retrieval-hit";
/// `OpenAPI` path for the repo projected-retrieval-hit endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_HIT_OPENAPI_PATH: &str = "/api/repo/projected-retrieval-hit";
/// Axum runtime path for the repo projected-retrieval-context endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_CONTEXT_AXUM_PATH: &str =
    "/api/repo/projected-retrieval-context";
/// `OpenAPI` path for the repo projected-retrieval-context endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_CONTEXT_OPENAPI_PATH: &str =
    "/api/repo/projected-retrieval-context";
/// Axum runtime path for the repo projected-page-family-context endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_AXUM_PATH: &str =
    "/api/repo/projected-page-family-context";
/// `OpenAPI` path for the repo projected-page-family-context endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_OPENAPI_PATH: &str =
    "/api/repo/projected-page-family-context";
/// Axum runtime path for the repo projected-page-family-search endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_AXUM_PATH: &str =
    "/api/repo/projected-page-family-search";
/// `OpenAPI` path for the repo projected-page-family-search endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_OPENAPI_PATH: &str =
    "/api/repo/projected-page-family-search";
/// Axum runtime path for the repo projected-page-family-cluster endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_AXUM_PATH: &str =
    "/api/repo/projected-page-family-cluster";
/// `OpenAPI` path for the repo projected-page-family-cluster endpoint.
pub const API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_OPENAPI_PATH: &str =
    "/api/repo/projected-page-family-cluster";
/// Axum runtime path for the repo projected-page-navigation endpoint.
pub const API_REPO_PROJECTED_PAGE_NAVIGATION_AXUM_PATH: &str =
    "/api/repo/projected-page-navigation";
/// `OpenAPI` path for the repo projected-page-navigation endpoint.
pub const API_REPO_PROJECTED_PAGE_NAVIGATION_OPENAPI_PATH: &str =
    "/api/repo/projected-page-navigation";
/// Axum runtime path for the repo projected-page-navigation-search endpoint.
pub const API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_AXUM_PATH: &str =
    "/api/repo/projected-page-navigation-search";
/// `OpenAPI` path for the repo projected-page-navigation-search endpoint.
pub const API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_OPENAPI_PATH: &str =
    "/api/repo/projected-page-navigation-search";
/// Axum runtime path for the repo projected-page-index-tree endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREE_AXUM_PATH: &str =
    "/api/repo/projected-page-index-tree";
/// `OpenAPI` path for the repo projected-page-index-tree endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREE_OPENAPI_PATH: &str =
    "/api/repo/projected-page-index-tree";
/// Axum runtime path for the repo projected-page-index-tree-search endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_AXUM_PATH: &str =
    "/api/repo/projected-page-index-tree-search";
/// `OpenAPI` path for the repo projected-page-index-tree-search endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_OPENAPI_PATH: &str =
    "/api/repo/projected-page-index-tree-search";
/// Axum runtime path for the repo projected-page-search endpoint.
pub const API_REPO_PROJECTED_PAGE_SEARCH_AXUM_PATH: &str = "/api/repo/projected-page-search";
/// `OpenAPI` path for the repo projected-page-search endpoint.
pub const API_REPO_PROJECTED_PAGE_SEARCH_OPENAPI_PATH: &str = "/api/repo/projected-page-search";
/// Axum runtime path for the repo projected-retrieval endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_AXUM_PATH: &str = "/api/repo/projected-retrieval";
/// `OpenAPI` path for the repo projected-retrieval endpoint.
pub const API_REPO_PROJECTED_RETRIEVAL_OPENAPI_PATH: &str = "/api/repo/projected-retrieval";
/// Axum runtime path for the repo projected-page-index-trees endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREES_AXUM_PATH: &str =
    "/api/repo/projected-page-index-trees";
/// `OpenAPI` path for the repo projected-page-index-trees endpoint.
pub const API_REPO_PROJECTED_PAGE_INDEX_TREES_OPENAPI_PATH: &str =
    "/api/repo/projected-page-index-trees";
