use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const REPO_OVERVIEW: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_OVERVIEW_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_OVERVIEW_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_MODULE_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_MODULE_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_MODULE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_SYMBOL_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_SYMBOL_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_SYMBOL_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_EXAMPLE_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_EXAMPLE_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_EXAMPLE_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_IMPORT_SEARCH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_IMPORT_SEARCH_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_IMPORT_SEARCH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_DOC_COVERAGE: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_DOC_COVERAGE_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_DOC_COVERAGE_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_INDEX_STATUS: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_INDEX_STATUS_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_INDEX_STATUS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const REPO_INDEX: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_INDEX_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_INDEX_OPENAPI_PATH,
    methods: &["post"],
    path_params: &[],
};

pub(super) const REPO_SYNC: RouteContract = RouteContract {
    axum_path: openapi_paths::API_REPO_SYNC_AXUM_PATH,
    openapi_path: openapi_paths::API_REPO_SYNC_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
