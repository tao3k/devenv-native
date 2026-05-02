//! VFS content-read route contract and metadata validation.

/// Stable route for the VFS content-read contract.
pub const VFS_CONTENT_ROUTE: &str = "/vfs/content";

/// Validate the stable VFS content-read request contract.
///
/// # Errors
///
/// Returns an error when the path is blank.
pub fn validate_vfs_content_request(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("VFS content requires a non-empty path".to_string());
    }
    Ok(())
}
