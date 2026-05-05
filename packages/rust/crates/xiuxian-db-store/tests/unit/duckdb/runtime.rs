use tempfile::tempdir;

use super::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig, must_err,
    must_ok, must_some, open_duckdb_connection, test_runtime,
};

#[test]
fn duckdb_connection_opens_in_memory_runtime() {
    let runtime = test_runtime("in-memory", DuckDbDatabasePath::InMemory);
    let connection = must_ok(
        DuckDbConnection::from_runtime(runtime),
        "in-memory DuckDB runtime should open",
    );

    let answer: i64 = must_ok(
        connection
            .connection()
            .query_row("SELECT 42", [], |row| row.get(0)),
        "DuckDB query should execute",
    );

    assert_eq!(answer, 42);
}

#[test]
fn duckdb_connection_creates_file_parent_and_temp_directory() {
    let root = must_ok(tempdir(), "create DuckDB file test root");
    let database_path = root.path().join("nested").join("store.duckdb");
    let temp_directory = root.path().join("tmp");
    let runtime = DuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::File(database_path.clone()),
        temp_directory: temp_directory.clone(),
        threads: 1,
        execution: DuckDbExecutionConfig {
            preserve_insertion_order: true,
            parquet_metadata_cache: false,
            prefer_virtual_arrow: true,
        },
        memory_limit: None,
        max_temp_directory_size: None,
        materialize_threshold_rows: 10,
    };

    let connection = must_ok(
        open_duckdb_connection(&runtime),
        "file DuckDB runtime should open",
    );
    must_ok(
        connection.execute_batch("CREATE TABLE storage_probe(value INTEGER);"),
        "DuckDB file connection should execute DDL",
    );

    assert!(must_some(database_path.parent(), "database parent").exists());
    assert!(temp_directory.exists());
}

#[test]
fn duckdb_connection_rejects_disabled_runtime() {
    let mut runtime = test_runtime("disabled", DuckDbDatabasePath::InMemory);
    runtime.enabled = false;

    let error = must_err(
        open_duckdb_connection(&runtime),
        "disabled runtime should fail",
    );

    assert_eq!(error, "DuckDB runtime is disabled");
}
