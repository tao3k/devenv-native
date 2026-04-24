use tempfile::tempdir;

use super::{
    DuckDbConnection, DuckDbDatabasePath, DuckDbExecutionConfig, DuckDbRuntimeConfig,
    build_duckdb_parquet_view_sql, build_duckdb_virtual_view_sql, ensure_duckdb_identifier,
    open_duckdb_connection,
};

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
    let connection =
        DuckDbConnection::from_runtime(runtime).expect("in-memory DuckDB runtime should open");

    let answer: i64 = connection
        .connection()
        .query_row("SELECT 42", [], |row| row.get(0))
        .expect("DuckDB query should execute");

    assert_eq!(answer, 42);
}

#[test]
fn duckdb_connection_creates_file_parent_and_temp_directory() {
    let root = tempdir().expect("create DuckDB file test root");
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

    let connection = open_duckdb_connection(&runtime).expect("file DuckDB runtime should open");
    connection
        .execute_batch("CREATE TABLE storage_probe(value INTEGER);")
        .expect("DuckDB file connection should execute DDL");

    assert!(database_path.parent().expect("database parent").exists());
    assert!(temp_directory.exists());
}

#[test]
fn duckdb_connection_rejects_disabled_runtime() {
    let mut runtime = test_runtime("disabled", DuckDbDatabasePath::InMemory);
    runtime.enabled = false;

    let error = open_duckdb_connection(&runtime).expect_err("disabled runtime should fail");

    assert_eq!(error, "DuckDB runtime is disabled");
}

#[test]
fn duckdb_sql_helpers_validate_and_escape_inputs() {
    assert!(ensure_duckdb_identifier("workflow_state", "table").is_ok());
    assert!(ensure_duckdb_identifier("9workflow", "table").is_err());

    let parquet_sql =
        build_duckdb_parquet_view_sql("workflow_state", std::path::Path::new("data's.parquet"))
            .expect("valid parquet view SQL");
    assert!(parquet_sql.contains("read_parquet('data''s.parquet')"));

    let virtual_sql = build_duckdb_virtual_view_sql("workflow_state", "ns'1", "arrow_relation")
        .expect("valid virtual view SQL");
    assert!(virtual_sql.contains("arrow_relation('ns''1', 'workflow_state')"));
}
