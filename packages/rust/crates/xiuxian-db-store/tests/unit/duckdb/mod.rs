use tempfile::tempdir;

use super::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig,
    DuckDbS3SecretConfig, DuckDbS3SecretProvider, DuckLakeAttachConfig, DuckLakeCatalog,
    DuckLakeDataPath, DuckLakeRecordBatchAppender, DuckLakeTableRef,
    append_ducklake_record_batches, attach_ducklake, build_duckdb_parquet_view_sql,
    build_duckdb_s3_secret_sql, build_duckdb_virtual_view_sql, build_ducklake_attach_sql,
    build_ducklake_extension_bootstrap_sql, build_ducklake_use_sql, ensure_duckdb_identifier,
    open_duckdb_connection,
};

mod ducklake;
mod runtime;
mod sql;

fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_err<T, E>(result: Result<T, E>, context: &str) -> E
where
    T: std::fmt::Debug,
{
    match result {
        Ok(value) => panic!("{context}: expected error, got {value:?}"),
        Err(error) => error,
    }
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

fn test_runtime(root_name: &str, database_path: DuckDbDatabasePath) -> DuckDbRuntimeConfig {
    let root = tempdir()
        .unwrap_or_else(|error| panic!("create DuckDB test tempdir `{root_name}`: {error}"));
    let root_path = root.keep();
    DuckDbRuntimeConfig {
        enabled: true,
        database_path,
        temp_directory: root_path.join("duckdb-tmp"),
        threads: 1,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: false,
            prefer_virtual_arrow: true,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: 10,
    }
}
