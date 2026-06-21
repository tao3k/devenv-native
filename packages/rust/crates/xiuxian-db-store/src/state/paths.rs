//! Project-local state path resolution.

use std::path::{Path, PathBuf};

use xiuxian_config_core::{ProjectDirs, resolve_cache_home};

use super::git_utils;

/// Default subdirectory used for shared state data inside the project
/// cache namespace.
pub const STATE_STORE_DIR_NAME: &str = "state";

/// Default DuckDB database filename for shared project-state records.
pub const STATE_STORE_DUCKDB_FILE_NAME: &str = "state.duckdb";

/// Named inputs for resolving the project cache namespace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectCacheRootConfig {
    /// Optional project root. When absent, current project dirs are resolved
    /// from the process environment.
    pub project_root: Option<PathBuf>,
    /// Optional cache home. When absent, `PRJ_CACHE_HOME` is used when set,
    /// otherwise `<git-toplevel>/.cache`.
    pub cache_home: Option<PathBuf>,
    /// Optional explicit namespace. When absent, the sanitized project-root
    /// directory name is used.
    pub project_namespace: Option<String>,
}

/// Resolve the namespaced project cache root for the current process.
#[must_use]
pub fn project_cache_root() -> PathBuf {
    project_cache_root_from_config(ProjectCacheRootConfig::default())
}

/// Resolve the shared project-state root for the current process.
#[must_use]
pub fn state_store_root() -> PathBuf {
    project_cache_root().join(STATE_STORE_DIR_NAME)
}

/// Resolve the shared project-state DuckDB path for the current process.
#[must_use]
pub fn state_store_duckdb_path() -> PathBuf {
    state_store_root().join(STATE_STORE_DUCKDB_FILE_NAME)
}

/// Resolve the namespaced project cache root from explicit inputs.
#[must_use]
pub fn project_cache_root_from_config(config: ProjectCacheRootConfig) -> PathBuf {
    let (project_root, cache_home) =
        project_root_and_cache_home(config.project_root, config.cache_home);
    let namespace = config
        .project_namespace
        .as_deref()
        .and_then(git_utils::sanitize_project_namespace)
        .unwrap_or_else(|| git_utils::project_namespace_from_root(&project_root));
    cache_home_with_namespace(cache_home, namespace.as_str())
}

fn project_root_and_cache_home(
    project_root: Option<PathBuf>,
    cache_home: Option<PathBuf>,
) -> (PathBuf, PathBuf) {
    match (project_root, cache_home) {
        (Some(project_root), Some(cache_home)) => (project_root, cache_home),
        (Some(project_root), None) => {
            let cache_home = resolve_cache_home(Some(&project_root))
                .unwrap_or_else(|| project_root.join(".cache"));
            (project_root, cache_home)
        }
        (None, Some(cache_home)) => {
            let project_root = git_utils::discover_git_toplevel_from_current_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            (project_root, cache_home)
        }
        (None, None) => {
            let dirs = ProjectDirs::from_env();
            (
                dirs.project_root_path().to_path_buf(),
                dirs.cache_home_path().to_path_buf(),
            )
        }
    }
}

fn cache_home_with_namespace(cache_home: PathBuf, namespace: &str) -> PathBuf {
    if path_file_name_eq(cache_home.as_path(), namespace) {
        cache_home
    } else {
        cache_home.join(namespace)
    }
}

fn path_file_name_eq(path: &Path, expected: &str) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value == expected)
}
