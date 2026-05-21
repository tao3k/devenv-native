//! `gateway::openapi::paths::vfs` owns Wendao openapi paths vfs behavior.

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
