use std::path::PathBuf;

use crate::settings::{merged_wendao_settings, wendao_config_file_override};
use xiuxian_config_core::resolve_project_root_or_cwd;
use xiuxian_wendao_runtime::config::{
    SearchDuckDbRuntimeConfig, resolve_search_duckdb_runtime_with_settings,
};

/// Resolve the current `search.duckdb` runtime configuration from merged
/// Wendao settings.
#[must_use]
pub fn resolve_search_duckdb_runtime() -> SearchDuckDbRuntimeConfig {
    let project_root = resolved_wendao_settings_root();
    let settings = merged_wendao_settings();
    resolve_search_duckdb_runtime_with_settings(project_root.as_path(), &settings)
}

fn resolved_wendao_settings_root() -> PathBuf {
    wendao_config_file_override()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(path)
            }
        })
        .filter(|path| path.is_file())
        .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_else(resolve_project_root_or_cwd)
}

#[cfg(feature = "duckdb")]
pub(crate) fn ensure_enabled_search_duckdb_runtime(
    runtime: SearchDuckDbRuntimeConfig,
    target: &str,
) -> Result<SearchDuckDbRuntimeConfig, String> {
    if !runtime.enabled {
        return Err(format!("search DuckDB runtime is disabled for `{target}`"));
    }
    Ok(runtime)
}

#[cfg(feature = "duckdb")]
pub(crate) fn resolve_enabled_search_duckdb_runtime(
    target: &str,
) -> Result<SearchDuckDbRuntimeConfig, String> {
    ensure_enabled_search_duckdb_runtime(resolve_search_duckdb_runtime(), target)
}
