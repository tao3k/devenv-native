//! Small git-root helpers for Artisan state fallback resolution.

use std::path::{Path, PathBuf};

/// Discover the nearest git toplevel from the current directory.
#[must_use]
pub fn discover_git_toplevel_from_current_dir() -> Option<PathBuf> {
    std::env::current_dir()
        .ok()
        .and_then(discover_git_toplevel_from)
}

/// Discover the nearest git toplevel by walking ancestors from `start`.
#[must_use]
pub fn discover_git_toplevel_from(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut cursor = start.as_ref();
    loop {
        if has_git_marker(cursor) {
            return Some(cursor.to_path_buf());
        }
        cursor = cursor.parent()?;
    }
}

/// Return true when `path` looks like a git worktree root.
#[must_use]
pub fn has_git_marker(path: impl AsRef<Path>) -> bool {
    let marker = path.as_ref().join(".git");
    marker.is_dir() || marker.is_file()
}
