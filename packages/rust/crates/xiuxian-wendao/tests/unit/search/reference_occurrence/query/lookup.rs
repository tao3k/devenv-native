//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[cfg(feature = "duckdb")]
use serial_test::serial;

use crate::search::SearchCorpusKind;

#[cfg(feature = "duckdb")]
use super::fixtures::write_search_duckdb_runtime_override;
use super::fixtures::{fixture_service, publish_reference_hits, sample_hit};
use crate::search::reference_occurrence::search_reference_occurrences;

#[tokio::test]
async fn reference_occurrence_query_reads_hits_from_published_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    publish_reference_hits(&service, "fp-1", &hits).await;

    let results = search_reference_occurrences(&service, "AlphaService", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "AlphaService");
    assert!(results[0].score > 0.0);

    let snapshot = service.status();
    let corpus = snapshot
        .corpora
        .iter()
        .find(|entry| entry.corpus == SearchCorpusKind::ReferenceOccurrence)
        .unwrap_or_else(|| panic!("reference occurrence corpus row should exist"));
    let telemetry = corpus
        .last_query_telemetry
        .as_ref()
        .unwrap_or_else(|| panic!("reference occurrence telemetry should be present"));
    assert_eq!(
        telemetry.source,
        crate::search::SearchQueryTelemetrySource::Scan
    );
    assert_eq!(telemetry.scope.as_deref(), Some("search"));
    assert!(telemetry.rows_scanned >= 1);
    assert!(telemetry.matched_rows >= 1);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn reference_occurrence_query_reads_hits_from_published_epoch_with_duckdb_query_engine() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/reference-occurrence-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("config override: {error}"));
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit("AlphaService", "src/lib.rs", 10),
        sample_hit("BetaThing", "src/beta.rs", 20),
    ];
    publish_reference_hits(&service, "fp-duckdb", &hits).await;

    let results = search_reference_occurrences(&service, "AlphaService", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "AlphaService");
    assert!(results[0].score > 0.0);
}
