#[cfg(feature = "duckdb")]
use super::*;

#[cfg(feature = "duckdb")]
#[test]
#[serial]
fn configured_search_duckdb_connection_opens_in_memory_runtime() -> TestResult {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/runtime-tmp"
threads = 2
preserve_insertion_order = false
parquet_metadata_cache = true
memory_limit = "3GB"
max_temp_directory_size = "11GB"
"#,
    )?;

    let connection =
        crate::duckdb::SearchDuckDbConnection::configured().map_err(std::io::Error::other)?;
    let mut settings = connection
        .connection()
        .prepare(
            "select
                current_setting('threads'),
                current_setting('temp_directory'),
                current_setting('preserve_insertion_order'),
                current_setting('parquet_metadata_cache')
            ",
        )
        .map_err(std::io::Error::other)?;
    let (threads, temp_directory, preserve_insertion_order, parquet_metadata_cache) = settings
        .query_row([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, bool>(2)?,
                row.get::<_, bool>(3)?,
            ))
        })
        .map_err(std::io::Error::other)?;
    connection
        .connection()
        .execute("create table ducks (name text)", [])
        .map_err(std::io::Error::other)?;
    assert_eq!(threads, 2);
    assert!(
        temp_directory.ends_with(".cache/duckdb/runtime-tmp"),
        "unexpected temp directory setting: {temp_directory}"
    );
    assert!(!preserve_insertion_order);
    assert!(parquet_metadata_cache);

    Ok(())
}

#[cfg(feature = "duckdb")]
#[test]
#[serial]
fn configured_parquet_query_engine_uses_duckdb_in_duckdb_build() -> TestResult {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = false
database_path = ":memory:"
temp_directory = ".cache/duckdb/repo-query-tmp"
threads = 2
"#,
    )?;

    let engine = ParquetQueryEngine::configured().map_err(std::io::Error::other)?;
    assert_eq!(engine.kind(), LocalRelationEngineKind::DuckDb);

    Ok(())
}
