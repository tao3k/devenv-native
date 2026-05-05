//! Project-local config, cache, data, and root path resolution helpers.

use std::path::{Path, PathBuf};

/// Resolve one optional env-style path-like value against `project_root`.
///
/// Blank values are treated as absent. Relative paths remain relative when no
/// project root is available.
#[must_use]
pub fn resolve_path_from_value(
    project_root: Option<&Path>,
    value: Option<&str>,
) -> Option<PathBuf> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else if let Some(root) = project_root {
                root.join(path)
            } else {
                path
            }
        })
}

/// Resolve project root from environment or git ancestry.
///
/// Resolution order:
/// 1. `PRJ_ROOT` (absolute or relative to current directory).
/// 2. Closest ancestor containing `.git`, starting from current directory.
///
/// Returns `None` when no current directory can be resolved.
#[must_use]
pub fn resolve_project_root() -> Option<PathBuf> {
    if let Some(path) = std::env::var("PRJ_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            return Some(candidate);
        }
        if let Ok(current_dir) = std::env::current_dir() {
            return Some(current_dir.join(candidate));
        }
        return None;
    }

    let mut cursor = std::env::current_dir().ok()?;
    loop {
        if cursor.join(".git").exists() {
            return Some(cursor);
        }
        if !cursor.pop() {
            break;
        }
    }
    None
}

/// Resolve config-home from `PRJ_CONFIG_HOME` or `<project_root>/.config`.
#[must_use]
pub fn resolve_config_home(project_root: Option<&Path>) -> Option<PathBuf> {
    resolve_home(project_root, "PRJ_CONFIG_HOME", ".config")
}

/// Resolve data-home from `PRJ_DATA_HOME` or `<project_root>/.data`.
#[must_use]
pub fn resolve_data_home(project_root: Option<&Path>) -> Option<PathBuf> {
    resolve_home(project_root, "PRJ_DATA_HOME", ".data")
}

/// Resolve cache-home from `PRJ_CACHE_HOME` or `<project_root>/.cache`.
#[must_use]
pub fn resolve_cache_home(project_root: Option<&Path>) -> Option<PathBuf> {
    resolve_home(project_root, "PRJ_CACHE_HOME", ".cache")
}

/// Resolve cache-home from one optional env-style value or `<project_root>/.cache`.
#[must_use]
pub fn resolve_cache_home_from_value(
    project_root: Option<&Path>,
    env_value: Option<&str>,
) -> Option<PathBuf> {
    resolve_home_from_value(project_root, env_value, ".cache")
}

/// Normalize an explicit `config_home` with optional `project_root`.
#[must_use]
pub fn normalize_config_home(
    project_root: Option<&Path>,
    config_home: Option<&Path>,
) -> Option<PathBuf> {
    match config_home {
        Some(path) if path.is_absolute() => Some(path.to_path_buf()),
        Some(path) => project_root.map(|root| root.join(path)),
        None => project_root.map(|root| root.join(".config")),
    }
}

/// Resolve project root with a stable fallback to current directory, then `"."`.
#[must_use]
pub fn resolve_project_root_or_cwd() -> PathBuf {
    resolve_project_root()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve project root from one optional raw env-style value and a cwd
/// fallback.
///
/// Blank values are treated as absent. Relative values are resolved against the
/// provided current directory when available.
#[must_use]
pub fn resolve_project_root_or_cwd_from_value(
    env_value: Option<&str>,
    current_dir: Option<&Path>,
) -> PathBuf {
    if let Some(path) = resolve_path_from_value(current_dir, env_value) {
        return path;
    }

    current_dir.map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// Convert `path` to absolute using `project_root` when needed.
#[must_use]
pub fn absolutize_path(project_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        project_root.join(path)
    }
}

fn resolve_home(
    project_root: Option<&Path>,
    env_key: &str,
    default_relative: &str,
) -> Option<PathBuf> {
    let env_value = std::env::var(env_key).ok();
    resolve_home_from_value(project_root, env_value.as_deref(), default_relative)
}

pub(crate) fn resolve_home_from_value(
    project_root: Option<&Path>,
    env_value: Option<&str>,
    default_relative: &str,
) -> Option<PathBuf> {
    resolve_path_from_value(project_root, env_value)
        .or_else(|| project_root.map(|root| root.join(default_relative)))
}
