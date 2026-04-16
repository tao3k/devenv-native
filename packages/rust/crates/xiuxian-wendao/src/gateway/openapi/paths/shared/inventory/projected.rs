use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const REPO_PROJECTED_PAGES: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGES_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_GAP_REPORT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_GAP_REPORT_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_GAP_REPORT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_INDEX_NODE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_NODE_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_NODE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_RETRIEVAL_HIT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_HIT_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_HIT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_RETRIEVAL_CONTEXT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_CONTEXT_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_FAMILY_CONTEXT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_FAMILY_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_FAMILY_CLUSTER: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_FAMILY_CLUSTER_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_NAVIGATION: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_NAVIGATION_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_NAVIGATION_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_NAVIGATION_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_NAVIGATION_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_INDEX_TREE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREE_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_RETRIEVAL: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_RETRIEVAL_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_PROJECTED_PAGE_INDEX_TREES: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREES_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_PROJECTED_PAGE_INDEX_TREES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
