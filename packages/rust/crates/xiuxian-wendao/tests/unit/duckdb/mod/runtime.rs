#[cfg(feature = "duckdb")]
use super::{
    DEFAULT_SEARCH_DUCKDB_THREADS, Path, load_toml_settings_from_path,
    resolve_search_duckdb_runtime_with_settings,
};
use super::{
    DuckDbDatabasePath, TestResult, resolve_search_duckdb_runtime, serial,
    write_search_duckdb_runtime_override,
};

#[test]
#[serial]
fn resolve_search_duckdb_runtime_reads_override_values() -> TestResult {
    let temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ".data/duckdb/search.db"
temp_directory = ".cache/duckdb/custom-tmp"
threads = 6
preserve_insertion_order = true
parquet_metadata_cache = false
memory_limit = "3GB"
max_temp_directory_size = "9GB"
materialize_threshold_rows = 123
prefer_virtual_arrow = false
"#,
    )?;

    let runtime = resolve_search_duckdb_runtime();
    assert!(runtime.enabled);
    assert_eq!(
        runtime.database_path,
        DuckDbDatabasePath::File(temp.path().join(".data/duckdb/search.db"))
    );
    assert_eq!(
        runtime.temp_directory,
        temp.path().join(".cache/duckdb/custom-tmp")
    );
    assert_eq!(runtime.threads, 6);
    assert!(runtime.execution.preserve_insertion_order);
    assert!(!runtime.execution.parquet_metadata_cache);
    assert_eq!(runtime.memory_limit.as_deref(), Some("3GB"));
    assert_eq!(runtime.max_temp_directory_size.as_deref(), Some("9GB"));
    assert_eq!(runtime.materialize_threshold_rows, 123);
    assert!(!runtime.execution.prefer_virtual_arrow);

    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
fn embedded_search_duckdb_defaults_follow_system_profile() -> TestResult {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_path = crate_root.join("resources/config/wendao.toml");
    let settings = load_toml_settings_from_path(resource_path.as_path())?;
    let runtime = resolve_search_duckdb_runtime_with_settings(crate_root, &settings);

    assert!(runtime.enabled);
    assert_eq!(runtime.database_path, DuckDbDatabasePath::InMemory);
    assert_eq!(runtime.temp_directory, crate_root.join(".cache/duckdb/tmp"));
    assert_eq!(runtime.threads, DEFAULT_SEARCH_DUCKDB_THREADS);
    assert!(runtime.execution.preserve_insertion_order);
    assert!(!runtime.execution.parquet_metadata_cache);
    assert_eq!(runtime.memory_limit, None);
    assert_eq!(runtime.max_temp_directory_size, None);

    Ok(())
}
