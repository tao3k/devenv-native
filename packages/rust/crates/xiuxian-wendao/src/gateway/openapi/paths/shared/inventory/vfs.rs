use crate::gateway::{self as openapi_paths, RouteContract};

pub(super) const VFS_ROOT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_VFS_ROOT_AXUM_PATH,
    openapi_path: openapi_paths::API_VFS_ROOT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const VFS_SCAN: RouteContract = RouteContract {
    axum_path: openapi_paths::API_VFS_SCAN_AXUM_PATH,
    openapi_path: openapi_paths::API_VFS_SCAN_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const VFS_CAT: RouteContract = RouteContract {
    axum_path: openapi_paths::API_VFS_CAT_AXUM_PATH,
    openapi_path: openapi_paths::API_VFS_CAT_OPENAPI_PATH,
    methods: &["get"],
    path_params: &[],
};

pub(super) const VFS_ENTRY: RouteContract = RouteContract {
    axum_path: openapi_paths::API_VFS_ENTRY_AXUM_PATH,
    openapi_path: openapi_paths::API_VFS_ENTRY_OPENAPI_PATH,
    methods: &["get"],
    path_params: &["path"],
};
