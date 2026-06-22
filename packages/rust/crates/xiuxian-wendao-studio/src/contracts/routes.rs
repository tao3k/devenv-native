//! Studio-owned HTTP route contracts and `OpenAPI` path inventory.

/// One declared route contract in the Wendao gateway surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteContract {
    /// The Axum runtime path pattern.
    pub axum_path: &'static str,
    /// The normalized `OpenAPI` path pattern.
    pub openapi_path: &'static str,
    /// Supported lowercase HTTP methods.
    pub methods: &'static [&'static str],
    /// Required `OpenAPI` path parameter names for this route.
    pub path_params: &'static [&'static str],
}

/// Axum runtime path for the health endpoint.
pub const API_HEALTH_AXUM_PATH: &str = "/api/health";
/// `OpenAPI` path for the health endpoint.
pub const API_HEALTH_OPENAPI_PATH: &str = "/api/health";
/// Axum runtime path for the stats endpoint.
pub const API_STATS_AXUM_PATH: &str = "/api/stats";
/// `OpenAPI` path for the stats endpoint.
pub const API_STATS_OPENAPI_PATH: &str = "/api/stats";
/// Axum runtime path for the notify endpoint.
pub const API_NOTIFY_AXUM_PATH: &str = "/api/notify";
/// `OpenAPI` path for the notify endpoint.
pub const API_NOTIFY_OPENAPI_PATH: &str = "/api/notify";
/// Axum runtime path for Gateway public API-token issuance.
pub const API_AUTH_TOKENS_AXUM_PATH: &str = "/v1/auth/tokens";
/// `OpenAPI` path for Gateway public API-token issuance.
pub const API_AUTH_TOKENS_OPENAPI_PATH: &str = "/v1/auth/tokens";

/// Axum runtime path for the docs projected-gap-report endpoint.
pub const API_DOCS_PROJECTED_GAP_REPORT_AXUM_PATH: &str = "/api/docs/projected-gap-report";
/// `OpenAPI` path for the docs projected-gap-report endpoint.
pub const API_DOCS_PROJECTED_GAP_REPORT_OPENAPI_PATH: &str = "/api/docs/projected-gap-report";
/// Axum runtime path for the docs planner-item endpoint.
pub const API_DOCS_PLANNER_ITEM_AXUM_PATH: &str = "/api/docs/planner-item";
/// `OpenAPI` path for the docs planner-item endpoint.
pub const API_DOCS_PLANNER_ITEM_OPENAPI_PATH: &str = "/api/docs/planner-item";
/// Axum runtime path for the docs planner-search endpoint.
pub const API_DOCS_PLANNER_SEARCH_AXUM_PATH: &str = "/api/docs/planner-search";
/// `OpenAPI` path for the docs planner-search endpoint.
pub const API_DOCS_PLANNER_SEARCH_OPENAPI_PATH: &str = "/api/docs/planner-search";
/// Axum runtime path for the docs planner-queue endpoint.
pub const API_DOCS_PLANNER_QUEUE_AXUM_PATH: &str = "/api/docs/planner-queue";
/// `OpenAPI` path for the docs planner-queue endpoint.
pub const API_DOCS_PLANNER_QUEUE_OPENAPI_PATH: &str = "/api/docs/planner-queue";
/// Axum runtime path for the docs planner-rank endpoint.
pub const API_DOCS_PLANNER_RANK_AXUM_PATH: &str = "/api/docs/planner-rank";
/// `OpenAPI` path for the docs planner-rank endpoint.
pub const API_DOCS_PLANNER_RANK_OPENAPI_PATH: &str = "/api/docs/planner-rank";
/// Axum runtime path for the docs planner-workset endpoint.
pub const API_DOCS_PLANNER_WORKSET_AXUM_PATH: &str = "/api/docs/planner-workset";
/// `OpenAPI` path for the docs planner-workset endpoint.
pub const API_DOCS_PLANNER_WORKSET_OPENAPI_PATH: &str = "/api/docs/planner-workset";
/// Axum runtime path for the docs search endpoint.
pub const API_DOCS_SEARCH_AXUM_PATH: &str = "/api/docs/search";
/// `OpenAPI` path for the docs search endpoint.
pub const API_DOCS_SEARCH_OPENAPI_PATH: &str = "/api/docs/search";
/// Axum runtime path for the docs retrieval endpoint.
pub const API_DOCS_RETRIEVAL_AXUM_PATH: &str = "/api/docs/retrieval";
/// `OpenAPI` path for the docs retrieval endpoint.
pub const API_DOCS_RETRIEVAL_OPENAPI_PATH: &str = "/api/docs/retrieval";
/// Axum runtime path for the docs retrieval-context endpoint.
pub const API_DOCS_RETRIEVAL_CONTEXT_AXUM_PATH: &str = "/api/docs/retrieval-context";
/// `OpenAPI` path for the docs retrieval-context endpoint.
pub const API_DOCS_RETRIEVAL_CONTEXT_OPENAPI_PATH: &str = "/api/docs/retrieval-context";
/// Axum runtime path for the docs retrieval-hit endpoint.
pub const API_DOCS_RETRIEVAL_HIT_AXUM_PATH: &str = "/api/docs/retrieval-hit";
/// `OpenAPI` path for the docs retrieval-hit endpoint.
pub const API_DOCS_RETRIEVAL_HIT_OPENAPI_PATH: &str = "/api/docs/retrieval-hit";
/// Axum runtime path for the docs page endpoint.
pub const API_DOCS_PAGE_AXUM_PATH: &str = "/api/docs/page";
/// `OpenAPI` path for the docs page endpoint.
pub const API_DOCS_PAGE_OPENAPI_PATH: &str = "/api/docs/page";
/// Axum runtime path for the docs page-index-tree endpoint.
pub const API_DOCS_PAGE_INDEX_TREE_AXUM_PATH: &str = "/api/docs/page-index-tree";
/// `OpenAPI` path for the docs page-index-tree endpoint.
pub const API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH: &str = "/api/docs/page-index-tree";
/// Axum runtime path for the docs family-context endpoint.
pub const API_DOCS_FAMILY_CONTEXT_AXUM_PATH: &str = "/api/docs/family-context";
/// `OpenAPI` path for the docs family-context endpoint.
pub const API_DOCS_FAMILY_CONTEXT_OPENAPI_PATH: &str = "/api/docs/family-context";
/// Axum runtime path for the docs family-search endpoint.
pub const API_DOCS_FAMILY_SEARCH_AXUM_PATH: &str = "/api/docs/family-search";
/// `OpenAPI` path for the docs family-search endpoint.
pub const API_DOCS_FAMILY_SEARCH_OPENAPI_PATH: &str = "/api/docs/family-search";
/// Axum runtime path for the docs family-cluster endpoint.
pub const API_DOCS_FAMILY_CLUSTER_AXUM_PATH: &str = "/api/docs/family-cluster";
/// `OpenAPI` path for the docs family-cluster endpoint.
pub const API_DOCS_FAMILY_CLUSTER_OPENAPI_PATH: &str = "/api/docs/family-cluster";
/// Axum runtime path for the docs navigation endpoint.
pub const API_DOCS_NAVIGATION_AXUM_PATH: &str = "/api/docs/navigation";
/// `OpenAPI` path for the docs navigation endpoint.
pub const API_DOCS_NAVIGATION_OPENAPI_PATH: &str = "/api/docs/navigation";
/// Axum runtime path for the docs navigation-search endpoint.
pub const API_DOCS_NAVIGATION_SEARCH_AXUM_PATH: &str = "/api/docs/navigation-search";
/// `OpenAPI` path for the docs navigation-search endpoint.
pub const API_DOCS_NAVIGATION_SEARCH_OPENAPI_PATH: &str = "/api/docs/navigation-search";

/// Axum runtime path for the 3D topology endpoint.
pub const API_TOPOLOGY_3D_AXUM_PATH: &str = "/api/topology/3d";
/// `OpenAPI` path for the 3D topology endpoint.
pub const API_TOPOLOGY_3D_OPENAPI_PATH: &str = "/api/topology/3d";

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

/// Axum runtime path for the search-plane status endpoint.
pub const API_SEARCH_INDEX_STATUS_AXUM_PATH: &str = "/api/search/index/status";
/// `OpenAPI` path for the search-plane status endpoint.
pub const API_SEARCH_INDEX_STATUS_OPENAPI_PATH: &str = "/api/search/index/status";

/// Axum runtime path for the UI capabilities endpoint.
pub const API_UI_CAPABILITIES_AXUM_PATH: &str = "/api/ui/capabilities";
/// `OpenAPI` path for the UI capabilities endpoint.
pub const API_UI_CAPABILITIES_OPENAPI_PATH: &str = "/api/ui/capabilities";
/// Axum runtime path for the generic plugin artifact inspection endpoint.
pub const API_UI_PLUGIN_ARTIFACT_AXUM_PATH: &str =
    "/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}";
/// `OpenAPI` path for the generic plugin artifact inspection endpoint.
pub const API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH: &str =
    "/api/ui/plugins/{plugin_id}/artifacts/{artifact_id}";

/// Axum runtime path for the VFS root endpoint.
pub const API_VFS_ROOT_AXUM_PATH: &str = "/api/vfs";
/// `OpenAPI` path for the VFS root endpoint.
pub const API_VFS_ROOT_OPENAPI_PATH: &str = "/api/vfs";
/// Axum runtime path for the VFS scan endpoint.
pub const API_VFS_SCAN_AXUM_PATH: &str = "/api/vfs/scan";
/// `OpenAPI` path for the VFS scan endpoint.
pub const API_VFS_SCAN_OPENAPI_PATH: &str = "/api/vfs/scan";
/// Axum runtime path for the VFS cat endpoint.
pub const API_VFS_CAT_AXUM_PATH: &str = "/api/vfs/cat";
/// `OpenAPI` path for the VFS cat endpoint.
pub const API_VFS_CAT_OPENAPI_PATH: &str = "/api/vfs/cat";
/// Axum runtime path for the VFS wildcard entry endpoint.
pub const API_VFS_ENTRY_AXUM_PATH: &str = "/api/vfs/{*path}";
/// `OpenAPI` path for the VFS entry endpoint.
pub const API_VFS_ENTRY_OPENAPI_PATH: &str = "/api/vfs/{path}";

macro_rules! route_contracts {
    ( $( $name:ident ),+ $(,)? ) => {
        &[$($name),+]
    };
}

const HEALTH: RouteContract = RouteContract {
    axum_path: API_HEALTH_AXUM_PATH,
    openapi_path: API_HEALTH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const STATS: RouteContract = RouteContract {
    axum_path: API_STATS_AXUM_PATH,
    openapi_path: API_STATS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const NOTIFY: RouteContract = RouteContract {
    axum_path: API_NOTIFY_AXUM_PATH,
    openapi_path: API_NOTIFY_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const AUTH_TOKENS: RouteContract = RouteContract {
    axum_path: API_AUTH_TOKENS_AXUM_PATH,
    openapi_path: API_AUTH_TOKENS_OPENAPI_PATH,
    methods: &["post"],
    path_params: &[],
};

const VFS_ROOT: RouteContract = RouteContract {
    axum_path: API_VFS_ROOT_AXUM_PATH,
    openapi_path: API_VFS_ROOT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const VFS_SCAN: RouteContract = RouteContract {
    axum_path: API_VFS_SCAN_AXUM_PATH,
    openapi_path: API_VFS_SCAN_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const VFS_CAT: RouteContract = RouteContract {
    axum_path: API_VFS_CAT_AXUM_PATH,
    openapi_path: API_VFS_CAT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const VFS_ENTRY: RouteContract = RouteContract {
    axum_path: API_VFS_ENTRY_AXUM_PATH,
    openapi_path: API_VFS_ENTRY_OPENAPI_PATH,
    methods: &["get"],
    path_params: &["path"],
};

const TOPOLOGY_3D: RouteContract = RouteContract {
    axum_path: API_TOPOLOGY_3D_AXUM_PATH,
    openapi_path: API_TOPOLOGY_3D_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const SEARCH_INDEX_STATUS: RouteContract = RouteContract {
    axum_path: API_SEARCH_INDEX_STATUS_AXUM_PATH,
    openapi_path: API_SEARCH_INDEX_STATUS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PROJECTED_GAP_REPORT: RouteContract = RouteContract {
    axum_path: API_DOCS_PROJECTED_GAP_REPORT_AXUM_PATH,
    openapi_path: API_DOCS_PROJECTED_GAP_REPORT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PLANNER_ITEM: RouteContract = RouteContract {
    axum_path: API_DOCS_PLANNER_ITEM_AXUM_PATH,
    openapi_path: API_DOCS_PLANNER_ITEM_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PLANNER_SEARCH: RouteContract = RouteContract {
    axum_path: API_DOCS_PLANNER_SEARCH_AXUM_PATH,
    openapi_path: API_DOCS_PLANNER_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PLANNER_QUEUE: RouteContract = RouteContract {
    axum_path: API_DOCS_PLANNER_QUEUE_AXUM_PATH,
    openapi_path: API_DOCS_PLANNER_QUEUE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PLANNER_RANK: RouteContract = RouteContract {
    axum_path: API_DOCS_PLANNER_RANK_AXUM_PATH,
    openapi_path: API_DOCS_PLANNER_RANK_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const PLANNER_WORKSET: RouteContract = RouteContract {
    axum_path: API_DOCS_PLANNER_WORKSET_AXUM_PATH,
    openapi_path: API_DOCS_PLANNER_WORKSET_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_SEARCH: RouteContract = RouteContract {
    axum_path: API_DOCS_SEARCH_AXUM_PATH,
    openapi_path: API_DOCS_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_RETRIEVAL: RouteContract = RouteContract {
    axum_path: API_DOCS_RETRIEVAL_AXUM_PATH,
    openapi_path: API_DOCS_RETRIEVAL_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_RETRIEVAL_CONTEXT: RouteContract = RouteContract {
    axum_path: API_DOCS_RETRIEVAL_CONTEXT_AXUM_PATH,
    openapi_path: API_DOCS_RETRIEVAL_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_RETRIEVAL_HIT: RouteContract = RouteContract {
    axum_path: API_DOCS_RETRIEVAL_HIT_AXUM_PATH,
    openapi_path: API_DOCS_RETRIEVAL_HIT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_PAGE: RouteContract = RouteContract {
    axum_path: API_DOCS_PAGE_AXUM_PATH,
    openapi_path: API_DOCS_PAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_PAGE_INDEX_TREE: RouteContract = RouteContract {
    axum_path: API_DOCS_PAGE_INDEX_TREE_AXUM_PATH,
    openapi_path: API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_FAMILY_CONTEXT: RouteContract = RouteContract {
    axum_path: API_DOCS_FAMILY_CONTEXT_AXUM_PATH,
    openapi_path: API_DOCS_FAMILY_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_FAMILY_SEARCH: RouteContract = RouteContract {
    axum_path: API_DOCS_FAMILY_SEARCH_AXUM_PATH,
    openapi_path: API_DOCS_FAMILY_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_FAMILY_CLUSTER: RouteContract = RouteContract {
    axum_path: API_DOCS_FAMILY_CLUSTER_AXUM_PATH,
    openapi_path: API_DOCS_FAMILY_CLUSTER_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_NAVIGATION: RouteContract = RouteContract {
    axum_path: API_DOCS_NAVIGATION_AXUM_PATH,
    openapi_path: API_DOCS_NAVIGATION_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const DOCS_NAVIGATION_SEARCH: RouteContract = RouteContract {
    axum_path: API_DOCS_NAVIGATION_SEARCH_AXUM_PATH,
    openapi_path: API_DOCS_NAVIGATION_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const UI_CAPABILITIES: RouteContract = RouteContract {
    axum_path: API_UI_CAPABILITIES_AXUM_PATH,
    openapi_path: API_UI_CAPABILITIES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const UI_PLUGIN_ARTIFACT: RouteContract = RouteContract {
    axum_path: API_UI_PLUGIN_ARTIFACT_AXUM_PATH,
    openapi_path: API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &["plugin_id", "artifact_id"],
};

const REPO_OVERVIEW: RouteContract = RouteContract {
    axum_path: API_REPO_OVERVIEW_AXUM_PATH,
    openapi_path: API_REPO_OVERVIEW_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_MODULE_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_MODULE_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_MODULE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_SYMBOL_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_SYMBOL_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_SYMBOL_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_EXAMPLE_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_EXAMPLE_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_EXAMPLE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_IMPORT_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_IMPORT_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_IMPORT_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_DOC_COVERAGE: RouteContract = RouteContract {
    axum_path: API_REPO_DOC_COVERAGE_AXUM_PATH,
    openapi_path: API_REPO_DOC_COVERAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_INDEX_STATUS: RouteContract = RouteContract {
    axum_path: API_REPO_INDEX_STATUS_AXUM_PATH,
    openapi_path: API_REPO_INDEX_STATUS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_INDEX: RouteContract = RouteContract {
    axum_path: API_REPO_INDEX_AXUM_PATH,
    openapi_path: API_REPO_INDEX_OPENAPI_PATH,
    methods: &["post"],
    path_params: &[],
};

const REPO_SYNC: RouteContract = RouteContract {
    axum_path: API_REPO_SYNC_AXUM_PATH,
    openapi_path: API_REPO_SYNC_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGES: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGES_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_GAP_REPORT: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_GAP_REPORT_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_GAP_REPORT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_INDEX_NODE: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_INDEX_NODE_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_INDEX_NODE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_RETRIEVAL_HIT: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_RETRIEVAL_HIT_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_RETRIEVAL_HIT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_RETRIEVAL_CONTEXT: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_RETRIEVAL_CONTEXT_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_RETRIEVAL_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_FAMILY_CONTEXT: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_FAMILY_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_FAMILY_CLUSTER: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_NAVIGATION: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_NAVIGATION_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_NAVIGATION_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_NAVIGATION_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_INDEX_TREE: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_INDEX_TREE_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_INDEX_TREE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_SEARCH: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_SEARCH_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_RETRIEVAL: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_RETRIEVAL_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_RETRIEVAL_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

const REPO_PROJECTED_PAGE_INDEX_TREES: RouteContract = RouteContract {
    axum_path: API_REPO_PROJECTED_PAGE_INDEX_TREES_AXUM_PATH,
    openapi_path: API_REPO_PROJECTED_PAGE_INDEX_TREES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

/// Stable inventory for the current Studio gateway route surface.
pub const WENDAO_GATEWAY_ROUTE_CONTRACTS: &[RouteContract] = route_contracts![
    HEALTH,
    STATS,
    NOTIFY,
    AUTH_TOKENS,
    VFS_ROOT,
    VFS_SCAN,
    VFS_CAT,
    VFS_ENTRY,
    TOPOLOGY_3D,
    SEARCH_INDEX_STATUS,
    PROJECTED_GAP_REPORT,
    PLANNER_ITEM,
    PLANNER_SEARCH,
    PLANNER_QUEUE,
    PLANNER_RANK,
    PLANNER_WORKSET,
    DOCS_SEARCH,
    DOCS_RETRIEVAL,
    DOCS_RETRIEVAL_CONTEXT,
    DOCS_RETRIEVAL_HIT,
    DOCS_PAGE,
    DOCS_PAGE_INDEX_TREE,
    DOCS_FAMILY_CONTEXT,
    DOCS_FAMILY_SEARCH,
    DOCS_FAMILY_CLUSTER,
    DOCS_NAVIGATION,
    DOCS_NAVIGATION_SEARCH,
    UI_CAPABILITIES,
    UI_PLUGIN_ARTIFACT,
    REPO_OVERVIEW,
    REPO_MODULE_SEARCH,
    REPO_SYMBOL_SEARCH,
    REPO_EXAMPLE_SEARCH,
    REPO_IMPORT_SEARCH,
    REPO_DOC_COVERAGE,
    REPO_INDEX_STATUS,
    REPO_INDEX,
    REPO_SYNC,
    REPO_PROJECTED_PAGES,
    REPO_PROJECTED_GAP_REPORT,
    REPO_PROJECTED_PAGE,
    REPO_PROJECTED_PAGE_INDEX_NODE,
    REPO_PROJECTED_RETRIEVAL_HIT,
    REPO_PROJECTED_RETRIEVAL_CONTEXT,
    REPO_PROJECTED_PAGE_FAMILY_CONTEXT,
    REPO_PROJECTED_PAGE_FAMILY_SEARCH,
    REPO_PROJECTED_PAGE_FAMILY_CLUSTER,
    REPO_PROJECTED_PAGE_NAVIGATION,
    REPO_PROJECTED_PAGE_NAVIGATION_SEARCH,
    REPO_PROJECTED_PAGE_INDEX_TREE,
    REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH,
    REPO_PROJECTED_PAGE_SEARCH,
    REPO_PROJECTED_RETRIEVAL,
    REPO_PROJECTED_PAGE_INDEX_TREES,
];
