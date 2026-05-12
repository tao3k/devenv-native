//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[cfg(feature = "duckdb")]
use serial_test::serial;

use crate::duckdb::ParquetQueryEngine;
use crate::search::SearchCorpusKind;
use crate::search::local_symbol::query::lookup::search_local_symbols;
use crate::search::local_symbol::query::shared::{
    decode_local_symbol_hits, execute_local_symbol_search, retained_window,
};
use crate::search::local_symbol::schema::local_symbol_batches;
use xiuxian_db_store::write_lance_batches_to_parquet_file;

#[cfg(feature = "duckdb")]
use super::fixtures::write_search_duckdb_runtime_override;
use super::fixtures::{fixture_service, publish_local_symbol_hits, sample_hit};

#[tokio::test]
async fn local_symbol_query_reads_hits_from_published_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaSymbol", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    publish_local_symbol_hits(&service, "fp-1", &hits).await;

    let results = search_local_symbols(&service, "alpha", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "AlphaSymbol");
    assert!(results[0].score > 0.0);

    let snapshot = service.status();
    let corpus = snapshot
        .corpora
        .iter()
        .find(|entry| entry.corpus == SearchCorpusKind::LocalSymbol)
        .unwrap_or_else(|| panic!("local symbol corpus row should exist"));
    let telemetry = corpus
        .last_query_telemetry
        .as_ref()
        .unwrap_or_else(|| panic!("local symbol telemetry should be present"));
    assert_eq!(
        telemetry.source,
        crate::search::SearchQueryTelemetrySource::Scan
    );
    assert_eq!(telemetry.scope.as_deref(), Some("search"));
    assert!(telemetry.rows_scanned >= 1);
    assert!(telemetry.matched_rows >= 1);
}

#[tokio::test]
async fn local_symbol_query_can_rerank_across_multiple_tables() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits_a = vec![sample_hit("AlphaSymbol", "src/lib.rs", 10)];
    let hits_b = vec![sample_hit("BetaAlphaHelper", "src/beta.rs", 20)];
    let batches_a =
        local_symbol_batches(&hits_a).unwrap_or_else(|error| panic!("batches a: {error}"));
    let batches_b =
        local_symbol_batches(&hits_b).unwrap_or_else(|error| panic!("batches b: {error}"));

    write_lance_batches_to_parquet_file(
        service
            .local_table_parquet_path(SearchCorpusKind::LocalSymbol, "local_symbol_project_a")
            .as_path(),
        &batches_a,
    )
    .unwrap_or_else(|error| panic!("write parquet a: {error}"));
    write_lance_batches_to_parquet_file(
        service
            .local_table_parquet_path(SearchCorpusKind::LocalSymbol, "local_symbol_project_b")
            .as_path(),
        &batches_b,
    )
    .unwrap_or_else(|error| panic!("write parquet b: {error}"));
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()
        .unwrap_or_else(|error| panic!("build parquet query engine: {error}"));
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(
            "local_symbol_project_a",
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, "local_symbol_project_a")
                .as_path(),
        )
        .await
        .unwrap_or_else(|error| panic!("register parquet a via query engine: {error}"));
    query_engine
        .ensure_parquet_table_registered(
            "local_symbol_project_b",
            service
                .local_table_parquet_path(SearchCorpusKind::LocalSymbol, "local_symbol_project_b")
                .as_path(),
        )
        .await
        .unwrap_or_else(|error| panic!("register parquet b via query engine: {error}"));

    let execution = execute_local_symbol_search(
        &query_engine,
        &[
            "local_symbol_project_a".to_string(),
            "local_symbol_project_b".to_string(),
        ],
        "alpha",
        retained_window(5),
    )
    .await
    .unwrap_or_else(|error| panic!("multi-table query should succeed: {error}"));

    let hits = decode_local_symbol_hits(&query_engine, execution.candidates)
        .await
        .unwrap_or_else(|error| panic!("decode hits should succeed: {error}"));
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].name, "AlphaSymbol");
    assert_eq!(hits[1].name, "BetaAlphaHelper");
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn local_symbol_query_reads_hits_from_published_epoch_with_duckdb_query_engine() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/local-symbol-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("write duckdb runtime override: {error}"));

    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaSymbol", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    publish_local_symbol_hits(&service, "fp-duckdb-search", &hits).await;

    let results = search_local_symbols(&service, "alpha", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "AlphaSymbol");
}
