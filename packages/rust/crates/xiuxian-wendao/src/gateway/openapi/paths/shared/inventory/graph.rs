use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const TOPOLOGY_3D: RouteContract = RouteContract {
    axum_path: openapi_paths::API_TOPOLOGY_3D_AXUM_PATH,
    openapi_path: openapi_paths::API_TOPOLOGY_3D_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};
