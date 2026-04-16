use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const UI_CAPABILITIES: RouteContract = RouteContract {
    axum_path: openapi_paths::API_UI_CAPABILITIES_AXUM_PATH,
    openapi_path: openapi_paths::API_UI_CAPABILITIES_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(crate) const UI_PLUGIN_ARTIFACT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_UI_PLUGIN_ARTIFACT_AXUM_PATH,
    openapi_path: openapi_paths::API_UI_PLUGIN_ARTIFACT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &["plugin_id", "artifact_id"],
};
