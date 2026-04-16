use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const HEALTH: RouteContract = RouteContract {
    axum_path: openapi_paths::API_HEALTH_AXUM_PATH,
    openapi_path: openapi_paths::API_HEALTH_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const STATS: RouteContract = RouteContract {
    axum_path: openapi_paths::API_STATS_AXUM_PATH,
    openapi_path: openapi_paths::API_STATS_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const NOTIFY: RouteContract = RouteContract {
    axum_path: openapi_paths::API_NOTIFY_AXUM_PATH,
    openapi_path: openapi_paths::API_NOTIFY_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
