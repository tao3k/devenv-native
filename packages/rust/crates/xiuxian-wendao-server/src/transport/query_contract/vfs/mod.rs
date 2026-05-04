//! VFS route contracts for Wendao Flight metadata.

mod content;
mod path;
mod resolve;
mod scan;

pub use content::{VFS_CONTENT_ROUTE, validate_vfs_content_request};
pub use path::WENDAO_VFS_PATH_HEADER;
pub use resolve::{VFS_RESOLVE_ROUTE, validate_vfs_resolve_request};
pub use scan::VFS_SCAN_ROUTE;
