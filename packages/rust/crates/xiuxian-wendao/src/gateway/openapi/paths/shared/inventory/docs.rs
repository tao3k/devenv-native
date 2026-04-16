use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const PROJECTED_GAP_REPORT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PROJECTED_GAP_REPORT_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PROJECTED_GAP_REPORT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const PLANNER_ITEM: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PLANNER_ITEM_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PLANNER_ITEM_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const PLANNER_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PLANNER_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PLANNER_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const PLANNER_QUEUE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PLANNER_QUEUE_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PLANNER_QUEUE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const PLANNER_RANK: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PLANNER_RANK_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PLANNER_RANK_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const PLANNER_WORKSET: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PLANNER_WORKSET_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PLANNER_WORKSET_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_RETRIEVAL: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_RETRIEVAL_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_RETRIEVAL_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_RETRIEVAL_CONTEXT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_RETRIEVAL_CONTEXT_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_RETRIEVAL_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_RETRIEVAL_HIT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_RETRIEVAL_HIT_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_RETRIEVAL_HIT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_PAGE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PAGE_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_PAGE_INDEX_TREE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_PAGE_INDEX_TREE_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_PAGE_INDEX_TREE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_FAMILY_CONTEXT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_FAMILY_CONTEXT_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_FAMILY_CONTEXT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_FAMILY_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_FAMILY_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_FAMILY_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_FAMILY_CLUSTER: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_FAMILY_CLUSTER_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_FAMILY_CLUSTER_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_NAVIGATION: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_NAVIGATION_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_NAVIGATION_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const DOCS_NAVIGATION_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_DOCS_NAVIGATION_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_DOCS_NAVIGATION_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
