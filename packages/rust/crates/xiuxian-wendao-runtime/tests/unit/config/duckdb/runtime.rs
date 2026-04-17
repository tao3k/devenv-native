use std::fs;

use super::{
    DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE, DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
    DEFAULT_SEARCH_DUCKDB_THREADS, DuckDbDatabasePath, resolve_search_duckdb_runtime_with_settings,
};
use crate::config::test_support;
use crate::config::{
    DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS, DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
    default_search_duckdb_temp_directory,
};

#[test]
fn duckdb_runtime_resolves_relative_paths_and_overrides() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let root = temp.path();
    let config_path = root.join("wendao.toml");
    fs::write(
        &config_path,
        r#"[search.duckdb]
enabled = true
database_path = ".data/duckdb/search.db"
temp_directory = ".cache/runtime-duckdb/tmp"
threads = 8
preserve_insertion_order = true
parquet_metadata_cache = false
memory_limit = "3GB"
max_temp_directory_size = "11GB"
materialize_threshold_rows = 12345
prefer_virtual_arrow = false
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_search_duckdb_runtime_with_settings(root, &settings);

    assert!(runtime.enabled);
    assert_eq!(
        runtime.database_path,
        DuckDbDatabasePath::File(root.join(".data/duckdb/search.db"))
    );
    assert_eq!(
        runtime.temp_directory,
        root.join(".cache/runtime-duckdb/tmp")
    );
    assert_eq!(runtime.threads, 8);
    assert!(runtime.execution.preserve_insertion_order);
    assert!(!runtime.execution.parquet_metadata_cache);
    assert_eq!(runtime.memory_limit.as_deref(), Some("3GB"));
    assert_eq!(runtime.max_temp_directory_size.as_deref(), Some("11GB"));
    assert_eq!(runtime.materialize_threshold_rows, 12345);
    assert!(!runtime.execution.prefer_virtual_arrow);

    Ok(())
}

#[test]
fn duckdb_runtime_falls_back_on_blank_or_invalid_values() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = tempfile::tempdir()?;
    let root = temp.path();
    let config_path = root.join("wendao.toml");
    fs::write(
        &config_path,
        r#"[search.duckdb]
enabled = false
database_path = "   "
temp_directory = "   "
threads = 0
memory_limit = "   "
max_temp_directory_size = "   "
materialize_threshold_rows = 0
prefer_virtual_arrow = true
"#,
    )?;

    let settings = test_support::load_test_settings_from_path(&config_path)?;
    let runtime = resolve_search_duckdb_runtime_with_settings(root, &settings);

    assert!(!runtime.enabled);
    assert_eq!(runtime.database_path, DuckDbDatabasePath::InMemory);
    assert_eq!(
        runtime.temp_directory,
        default_search_duckdb_temp_directory(root)
    );
    assert_eq!(runtime.threads, DEFAULT_SEARCH_DUCKDB_THREADS);
    assert_eq!(
        runtime.execution.preserve_insertion_order,
        DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER
    );
    assert_eq!(
        runtime.execution.parquet_metadata_cache,
        DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE
    );
    assert_eq!(runtime.memory_limit, None);
    assert_eq!(runtime.max_temp_directory_size, None);
    assert_eq!(
        runtime.materialize_threshold_rows,
        DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS
    );
    assert_eq!(
        runtime.execution.prefer_virtual_arrow,
        DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW
    );

    Ok(())
}
