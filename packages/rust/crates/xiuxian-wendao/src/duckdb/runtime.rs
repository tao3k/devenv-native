//! `duckdb::runtime` owns Wendao duckdb runtime behavior.

use std::path::PathBuf;

use crate::settings::{merged_wendao_settings, wendao_config_file_override};
#[cfg(feature = "duckdb")]
use std::path::Path;
use xiuxian_config_core::resolve_project_root_or_cwd;
#[cfg(feature = "duckdb")]
use xiuxian_wendao_runtime::config::DuckDbDatabasePath;
use xiuxian_wendao_runtime::config::SearchDuckDbRuntimeConfig;
use xiuxian_wendao_runtime::config::resolve_search_duckdb_runtime_with_settings;

/// Resolve the current `search.duckdb` runtime configuration from merged
/// Wendao settings.
#[must_use]
pub fn resolve_search_duckdb_runtime() -> SearchDuckDbRuntimeConfig {
    let has_config_override = wendao_config_file_override().is_some();
    let project_root = resolved_wendao_settings_root();
    let settings = merged_wendao_settings();
    let runtime = resolve_search_duckdb_runtime_with_settings(project_root.as_path(), &settings);
    #[cfg(test)]
    {
        isolate_default_search_duckdb_runtime_for_tests(runtime, has_config_override)
    }
    #[cfg(not(test))]
    {
        let _ = has_config_override;
        runtime
    }
}

#[cfg(feature = "duckdb")]
pub(crate) fn resolve_search_duckdb_runtime_for_storage_root(
    storage_root: &Path,
) -> SearchDuckDbRuntimeConfig {
    let mut runtime = resolve_search_duckdb_runtime();
    let runtime_root = storage_root.join("_runtime").join("duckdb");
    runtime.database_path = DuckDbDatabasePath::File(runtime_root.join("search.duckdb"));
    runtime.temp_directory = runtime_root.join("tmp");
    runtime
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

#[cfg(test)]
fn isolate_default_search_duckdb_runtime_for_tests(
    mut runtime: SearchDuckDbRuntimeConfig,
    has_config_override: bool,
) -> SearchDuckDbRuntimeConfig {
    if has_config_override {
        return runtime;
    }
    if !matches!(runtime.database_path, DuckDbDatabasePath::File(_)) {
        return runtime;
    }

    let runtime_root = std::env::temp_dir()
        .join("xiuxian-wendao-test-duckdb")
        .join(format!(
            "pid-{}-{}",
            std::process::id(),
            test_thread_id_fragment()
        ));
    runtime.database_path = DuckDbDatabasePath::File(runtime_root.join("search.duckdb"));
    runtime.temp_directory = runtime_root.join("tmp");
    runtime
}

#[cfg(test)]
fn test_thread_id_fragment() -> String {
    format!("{:?}", std::thread::current().id())
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .collect()
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
