use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tempfile::tempdir;
use xiuxian_vector::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
    write_lance_batches_to_parquet_file,
};
use xiuxian_wendao_runtime::config::{
    DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS, DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE,
    DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW, DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER,
};

use super::DuckDbParquetQueryEngine;
use crate::duckdb::{DuckDbDatabasePath, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig};

#[test]
fn repeated_parquet_view_registration_reuses_cached_entry_for_same_path() {
    let temp = tempdir().expect("tempdir should succeed");
    let parquet_path = temp.path().join("bench.parquet");
    write_test_parquet(parquet_path.as_path());
    let engine = DuckDbParquetQueryEngine::from_runtime(in_memory_runtime(temp.path()))
        .expect("duckdb engine should open");

    engine
        .register_parquet_view("bench_docs", parquet_path.as_path())
        .expect("initial parquet view registration should succeed");
    engine
        .register_parquet_view("bench_docs", parquet_path.as_path())
        .expect("repeated parquet view registration should succeed");

    let guard = engine.lock_runtime().expect("runtime lock should succeed");
    assert_eq!(guard.registered_parquet_views.len(), 1);
    assert_eq!(
        guard.registered_parquet_views.get("bench_docs"),
        Some(&parquet_path)
    );
}

#[test]
fn repeated_parquet_queries_return_readable_batches_on_one_registered_view() {
    let temp = tempdir().expect("tempdir should succeed");
    let parquet_path = temp.path().join("bench.parquet");
    write_test_parquet(parquet_path.as_path());
    let engine = DuckDbParquetQueryEngine::from_runtime(in_memory_runtime(temp.path()))
        .expect("duckdb engine should open");
    engine
        .register_parquet_view("bench_docs", parquet_path.as_path())
        .expect("parquet view registration should succeed");

    let first = engine
        .query_batches("select path from bench_docs")
        .expect("first parquet query should succeed");
    let second = engine
        .query_batches("select path from bench_docs")
        .expect("second parquet query should succeed");

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].num_rows(), 1);
    assert_eq!(second[0].num_rows(), 1);
}

fn in_memory_runtime(root: &Path) -> SearchDuckDbRuntimeConfig {
    SearchDuckDbRuntimeConfig {
        enabled: true,
        database_path: DuckDbDatabasePath::InMemory,
        temp_directory: root.join(".cache/duckdb-test/tmp"),
        threads: 2,
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

fn write_test_parquet(path: &Path) {
    let schema = Arc::new(LanceSchema::new_with_metadata(
        vec![LanceField::new("path", LanceDataType::Utf8, false)],
        HashMap::from([("domain".to_string(), "bench_docs".to_string())]),
    ));
    let batch = LanceRecordBatch::try_new(
        schema,
        vec![Arc::new(LanceStringArray::from(vec![
            "src/module.jl".to_string(),
        ]))],
    )
    .expect("record batch should build");
    write_lance_batches_to_parquet_file(path, &[batch])
        .expect("parquet fixture should write successfully");
}
