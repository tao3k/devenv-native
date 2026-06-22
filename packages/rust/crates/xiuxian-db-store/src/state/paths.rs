//! User-local Artisan state path resolution.

use std::path::{Path, PathBuf};

use super::git_utils;

/// Default user-home directory used for shared Artisan state data.
pub const ARTISAN_STATE_ROOT_DIR_NAME: &str = ".xiuxian-artisan-workshop";
/// Default subdirectory used for shared state data inside the Artisan state
/// namespace.
pub const STATE_STORE_DIR_NAME: &str = "state";

/// Default `DuckDB` database filename for shared state records.
pub const STATE_STORE_DUCKDB_FILE_NAME: &str = "state.duckdb";

/// Named inputs for resolving the shared Artisan state root.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtisanStateRootConfig {
    /// Optional project root. Used only to resolve relative explicit roots and
    /// as a fallback when no user home can be resolved.
    pub project_root: Option<PathBuf>,
    /// Optional explicit state root. Absolute values are used as-is; relative
    /// values are resolved against `project_root` when available.
    pub state_root: Option<PathBuf>,
    /// Optional home directory. When absent, `HOME` is used.
    pub home_dir: Option<PathBuf>,
}

/// Resolve the shared Artisan state root for the current process.
#[must_use]
pub fn artisan_state_root() -> PathBuf {
    artisan_state_root_from_config(ArtisanStateRootConfig::default())
}

/// Resolve the shared state-store root for the current process.
#[must_use]
pub fn state_store_root() -> PathBuf {
    artisan_state_root().join(STATE_STORE_DIR_NAME)
}

/// Resolve the shared state-store `DuckDB` path for the current process.
#[must_use]
pub fn state_store_duckdb_path() -> PathBuf {
    state_store_root().join(STATE_STORE_DUCKDB_FILE_NAME)
}

/// Resolve the shared Artisan state root from explicit inputs.
#[must_use]
pub fn artisan_state_root_from_config(config: ArtisanStateRootConfig) -> PathBuf {
    if let Some(state_root) = config
        .state_root
        .filter(|path| !path.as_os_str().is_empty())
    {
        return normalize_explicit_state_root(config.project_root.as_deref(), state_root);
    }

    let home_dir = config
        .home_dir
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(home_dir_from_env);
    if let Some(home_dir) = home_dir {
        return home_dir.join(ARTISAN_STATE_ROOT_DIR_NAME);
    }

    fallback_project_root(config.project_root).join(ARTISAN_STATE_ROOT_DIR_NAME)
}

fn normalize_explicit_state_root(project_root: Option<&Path>, state_root: PathBuf) -> PathBuf {
    if state_root.is_absolute() {
        state_root
    } else if let Some(project_root) = project_root {
        project_root.join(state_root)
    } else {
        state_root
    }
}

fn home_dir_from_env() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn fallback_project_root(project_root: Option<PathBuf>) -> PathBuf {
    project_root
        .or_else(git_utils::discover_git_toplevel_from_current_dir)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."))
}
