//! Small git-root helpers for project-local state storage.

use std::path::{Path, PathBuf};

/// Fallback namespace used when a project root cannot provide a usable name.
pub const FALLBACK_PROJECT_CACHE_NAMESPACE: &str = "workspace";

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

/// Derive a stable cache namespace from a repository root path.
#[must_use]
pub fn project_namespace_from_root(root: impl AsRef<Path>) -> String {
    root.as_ref()
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(sanitize_project_namespace)
        .unwrap_or_else(|| FALLBACK_PROJECT_CACHE_NAMESPACE.to_owned())
}

/// Sanitize one user or filesystem provided project namespace.
#[must_use]
pub fn sanitize_project_namespace(value: &str) -> Option<String> {
    let mut normalized = String::new();
    let mut last_was_separator = false;
    for character in value.trim().chars() {
        let next = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
            character
        } else if character.is_whitespace() || matches!(character, '/' | '\\' | ':' | '@') {
            '-'
        } else {
            continue;
        };
        if next == '-' {
            if last_was_separator {
                continue;
            }
            last_was_separator = true;
        } else {
            last_was_separator = false;
        }
        normalized.push(next);
    }
    let trimmed = normalized.trim_matches(['-', '.']).to_string();
    (!trimmed.is_empty()).then_some(trimmed)
}
