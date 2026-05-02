//! `DuckDB` runtime configuration defaults for Wendao host behavior.

use std::path::{Path, PathBuf};

use crate::settings::{first_non_empty, get_setting_bool, get_setting_string, parse_positive_u64};
use serde_yaml::Value;
use xiuxian_config_core::{resolve_cache_home, resolve_path_from_value};
pub use xiuxian_db_store::duckdb::DuckDbDatabasePath;
use xiuxian_db_store::duckdb::{DuckDbExecutionConfig, DuckDbRuntimeConfig};

/// `DuckDB`'s special marker for one ephemeral in-process database.
///
/// This is a DuckDB-local catalog mode, not Wendao memory-layer state and not
/// an integration point for `xiuxian-memory-engine`.
pub const DEFAULT_SEARCH_DUCKDB_DATABASE_PATH: &str = ":memory:";
/// Default thread budget for bounded `DuckDB` analytics.
pub const DEFAULT_SEARCH_DUCKDB_THREADS: u64 = 4;
/// Default row-order policy for Wendao's bounded `DuckDB` search lane.
///
/// Keep `DuckDB`'s documented default and let workload-specific benchmarks opt
/// out explicitly when row-order preservation is provably unnecessary.
pub const DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER: bool = true;
/// Default Parquet metadata cache policy for Wendao's bounded `DuckDB` search
/// lane.
///
/// Keep `DuckDB`'s documented default and expose the setting for explicit
/// opt-in when repeated scans measurably benefit from metadata caching.
pub const DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE: bool = false;
/// Default row threshold for deciding when bounded materialization is worth it.
pub const DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS: u64 = 200_000;
/// Default preference for Arrow virtual-table registration.
pub const DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW: bool = true;

/// Runtime-owned `DuckDB` execution toggles for bounded Wendao search analytics.
pub type SearchDuckDbExecutionConfig = DuckDbExecutionConfig;

/// Runtime-owned `DuckDB` config for bounded Wendao search analytics.
pub type SearchDuckDbRuntimeConfig = DuckDbRuntimeConfig;

/// Resolve the default temp directory for bounded `DuckDB` analytics.
#[must_use]
pub fn default_search_duckdb_temp_directory(project_root: &Path) -> PathBuf {
    resolve_cache_home(Some(project_root))
        .unwrap_or_else(|| project_root.join(".cache"))
        .join("duckdb")
        .join("tmp")
}

fn default_search_duckdb_runtime(project_root: &Path) -> SearchDuckDbRuntimeConfig {
    SearchDuckDbRuntimeConfig {
        enabled: false,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: default_search_duckdb_temp_directory(project_root),
        threads: DEFAULT_SEARCH_DUCKDB_THREADS,
        execution: SearchDuckDbExecutionConfig {
            preserve_insertion_order: DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
            parquet_metadata_cache: DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
            prefer_virtual_arrow: DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
    }
}

fn resolve_non_empty_string(settings: &Value, dotted_key: &str) -> Option<String> {
    first_non_empty(&[get_setting_string(settings, dotted_key)])
}

fn resolve_database_path(project_root: &Path, raw: &str) -> DuckDbDatabasePath {
    if raw.trim() == DEFAULT_SEARCH_DUCKDB_DATABASE_PATH {
        DuckDbDatabasePath::InMemory
    } else {
        resolve_path_from_value(Some(project_root), Some(raw))
            .map_or(DuckDbDatabasePath::InMemory, DuckDbDatabasePath::File)
    }
}

/// Resolve `search.duckdb` from merged Wendao settings.
#[must_use]
pub fn resolve_search_duckdb_runtime_with_settings(
    project_root: &Path,
    settings: &Value,
) -> SearchDuckDbRuntimeConfig {
    let mut resolved = default_search_duckdb_runtime(project_root);

    if let Some(enabled) = get_setting_bool(settings, "search.duckdb.enabled") {
        resolved.enabled = enabled;
    }

    if let Some(database_path) = resolve_non_empty_string(settings, "search.duckdb.database_path") {
        resolved.database_path = resolve_database_path(project_root, &database_path);
    }

    if let Some(temp_directory) = resolve_non_empty_string(settings, "search.duckdb.temp_directory")
        .and_then(|value| resolve_path_from_value(Some(project_root), Some(value.as_str())))
    {
        resolved.temp_directory = temp_directory;
    }

    if let Some(threads) = resolve_non_empty_string(settings, "search.duckdb.threads")
        .as_deref()
        .and_then(parse_positive_u64)
    {
        resolved.threads = threads;
    }

    if let Some(preserve_insertion_order) =
        get_setting_bool(settings, "search.duckdb.preserve_insertion_order")
    {
        resolved.execution.preserve_insertion_order = preserve_insertion_order;
    }

    if let Some(parquet_metadata_cache) =
        get_setting_bool(settings, "search.duckdb.parquet_metadata_cache")
    {
        resolved.execution.parquet_metadata_cache = parquet_metadata_cache;
    }

    if let Some(memory_limit) = resolve_non_empty_string(settings, "search.duckdb.memory_limit") {
        resolved.memory_limit = Some(memory_limit);
    }

    if let Some(max_temp_directory_size) =
        resolve_non_empty_string(settings, "search.duckdb.max_temp_directory_size")
    {
        resolved.max_temp_directory_size = Some(max_temp_directory_size);
    }

    if let Some(threshold) =
        resolve_non_empty_string(settings, "search.duckdb.materialize_threshold_rows")
            .as_deref()
            .and_then(parse_positive_u64)
    {
        resolved.materialize_threshold_rows = threshold;
    }

    if let Some(prefer_virtual_arrow) =
        get_setting_bool(settings, "search.duckdb.prefer_virtual_arrow")
    {
        resolved.execution.prefer_virtual_arrow = prefer_virtual_arrow;
    }

    resolved
}

#[cfg(test)]
#[path = "../../../tests/unit/config/duckdb/runtime.rs"]
mod tests;
