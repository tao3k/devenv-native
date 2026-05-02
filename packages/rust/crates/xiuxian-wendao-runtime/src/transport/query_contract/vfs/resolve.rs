//! VFS resolve route contract and metadata validation.

/// Stable route for the VFS navigation-resolution contract.
pub const VFS_RESOLVE_ROUTE: &str = "/vfs/resolve";

/// Validate the stable VFS navigation-resolution request contract.
///
/// # Errors
///
/// Returns an error when the path is blank.
pub fn validate_vfs_resolve_request(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("VFS resolve requires a non-empty path".to_string());
    }
    Ok(())
}
