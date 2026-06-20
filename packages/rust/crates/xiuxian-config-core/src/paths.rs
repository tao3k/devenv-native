//! Project-local config, cache, data, runtime, and root path resolution helpers.

use std::path::{Path, PathBuf};

/// Resolved project-local directories derived from the `PRJ_*` environment
/// contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDirs {
    project_root: PathBuf,
    config_home: PathBuf,
    data_home: PathBuf,
    cache_home: PathBuf,
    runtime_dir: PathBuf,
}

/// Named inputs for resolving project-local directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDirsConfig {
    /// Project root used to resolve relative `PRJ_*` values.
    pub project_root: PathBuf,
    /// Optional `PRJ_CONFIG_HOME` value.
    pub config_home: Option<String>,
    /// Optional `PRJ_DATA_HOME` value.
    pub data_home: Option<String>,
    /// Optional `PRJ_CACHE_HOME` value.
    pub cache_home: Option<String>,
    /// Optional `PRJ_RUNTIME_DIR` value.
    pub runtime_dir: Option<String>,
}

impl ProjectDirsConfig {
    /// Build a config using only the project root and default homes.
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
            config_home: None,
            data_home: None,
            cache_home: None,
            runtime_dir: None,
        }
    }
}

impl ProjectDirs {
    /// Resolve project directories from the current process environment.
    #[must_use]
    pub fn from_env() -> Self {
        let project_root = resolve_project_root_or_cwd();
        Self::from_values(ProjectDirsConfig {
            project_root,
            config_home: std::env::var("PRJ_CONFIG_HOME").ok(),
            data_home: std::env::var("PRJ_DATA_HOME").ok(),
            cache_home: std::env::var("PRJ_CACHE_HOME").ok(),
            runtime_dir: std::env::var("PRJ_RUNTIME_DIR").ok(),
        })
    }

    /// Build project directories from explicit values.
    ///
    /// Blank values are treated as absent. Relative values are resolved against
    /// `project_root`, matching the `PRJ_*` environment contract.
    #[must_use]
    pub fn from_values(config: ProjectDirsConfig) -> Self {
        let ProjectDirsConfig {
            project_root,
            config_home,
            data_home,
            cache_home,
            runtime_dir,
        } = config;
        let config_home =
            resolve_home_from_value(Some(&project_root), config_home.as_deref(), ".config")
                .unwrap_or_else(|| project_root.join(".config"));
        let data_home = resolve_home_from_value(Some(&project_root), data_home.as_deref(), ".data")
            .unwrap_or_else(|| project_root.join(".data"));
        let cache_home =
            resolve_home_from_value(Some(&project_root), cache_home.as_deref(), ".cache")
                .unwrap_or_else(|| project_root.join(".cache"));
        let runtime_dir =
            resolve_home_from_value(Some(&project_root), runtime_dir.as_deref(), ".run")
                .unwrap_or_else(|| project_root.join(".run"));

        Self {
            project_root,
            config_home,
            data_home,
            cache_home,
            runtime_dir,
        }
    }

    /// Resolve `PRJ_ROOT`, falling back to the current directory and then `"."`.
    #[inline]
    #[must_use]
    pub fn project_root() -> PathBuf {
        Self::from_env().project_root
    }

    /// Resolve `PRJ_CONFIG_HOME`, defaulting to `<project_root>/.config`.
    #[inline]
    #[must_use]
    pub fn config_home() -> PathBuf {
        Self::from_env().config_home
    }

    /// Resolve `PRJ_DATA_HOME`, defaulting to `<project_root>/.data`.
    #[inline]
    #[must_use]
    pub fn data_home() -> PathBuf {
        Self::from_env().data_home
    }

    /// Resolve `PRJ_CACHE_HOME`, defaulting to `<project_root>/.cache`.
    #[inline]
    #[must_use]
    pub fn cache_home() -> PathBuf {
        Self::from_env().cache_home
    }

    /// Resolve `PRJ_RUNTIME_DIR`, defaulting to `<project_root>/.run`.
    #[inline]
    #[must_use]
    pub fn runtime_dir() -> PathBuf {
        Self::from_env().runtime_dir
    }

    /// Borrow the resolved project root.
    #[must_use]
    pub fn project_root_path(&self) -> &Path {
        &self.project_root
    }

    /// Borrow the resolved config home.
    #[must_use]
    pub fn config_home_path(&self) -> &Path {
        &self.config_home
    }

    /// Borrow the resolved data home.
    #[must_use]
    pub fn data_home_path(&self) -> &Path {
        &self.data_home
    }

    /// Borrow the resolved cache home.
    #[must_use]
    pub fn cache_home_path(&self) -> &Path {
        &self.cache_home
    }

    /// Borrow the resolved runtime directory.
    #[must_use]
    pub fn runtime_dir_path(&self) -> &Path {
        &self.runtime_dir
    }
}

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

/// Resolve runtime-dir from `PRJ_RUNTIME_DIR` or `<project_root>/.run`.
#[must_use]
pub fn resolve_runtime_dir(project_root: Option<&Path>) -> Option<PathBuf> {
    resolve_home(project_root, "PRJ_RUNTIME_DIR", ".run")
}

/// Resolve cache-home from one optional env-style value or `<project_root>/.cache`.
#[must_use]
pub fn resolve_cache_home_from_value(
    project_root: Option<&Path>,
    env_value: Option<&str>,
) -> Option<PathBuf> {
    resolve_home_from_value(project_root, env_value, ".cache")
}

/// Resolve runtime-dir from one optional env-style value or `<project_root>/.run`.
#[must_use]
pub fn resolve_runtime_dir_from_value(
    project_root: Option<&Path>,
    env_value: Option<&str>,
) -> Option<PathBuf> {
    resolve_home_from_value(project_root, env_value, ".run")
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
