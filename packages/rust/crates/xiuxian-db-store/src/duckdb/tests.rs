use tempfile::tempdir;

use super::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig,
    build_duckdb_parquet_view_sql, build_duckdb_virtual_view_sql, ensure_duckdb_identifier,
    open_duckdb_connection,
};

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

#[test]
fn duckdb_sql_helpers_validate_and_escape_inputs() {
    assert!(ensure_duckdb_identifier("workflow_state", "table").is_ok());
    assert!(ensure_duckdb_identifier("9workflow", "table").is_err());

    let parquet_sql = must_ok(
        build_duckdb_parquet_view_sql("workflow_state", std::path::Path::new("data's.parquet")),
        "valid parquet view SQL",
    );
    assert!(parquet_sql.contains("read_parquet('data''s.parquet')"));

    let virtual_sql = must_ok(
        build_duckdb_virtual_view_sql("workflow_state", "ns'1", "arrow_relation"),
        "valid virtual view SQL",
    );
    assert!(virtual_sql.contains("arrow_relation('ns''1', 'workflow_state')"));
}
