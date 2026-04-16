use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const SEARCH_INDEX_STATUS: RouteContract = RouteContract {
    axum_path: openapi_paths::API_SEARCH_INDEX_STATUS_AXUM_PATH,
    openapi_path: openapi_paths::API_SEARCH_INDEX_STATUS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
