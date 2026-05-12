//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[cfg(feature = "duckdb")]
use serial_test::serial;

use crate::search::attachment::query::lookup::search_attachment_hits;

#[cfg(feature = "duckdb")]
use super::fixtures::write_search_duckdb_runtime_override;
use super::fixtures::{fixture_service, publish_attachment_hits, sample_hit};

#[tokio::test]
async fn attachment_query_reads_hits_from_published_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit(
            "topology.png",
            "docs/alpha.md",
            "assets/topology.png",
            "image",
        ),
        sample_hit("spec.pdf", "docs/alpha.md", "files/spec.pdf", "pdf"),
    ];
    publish_attachment_hits(&service, "fp-1", &hits).await;

    let results = search_attachment_hits(&service, "topology", 5, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].attachment_name, "topology.png");
    assert!(results[0].score > 0.0);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn attachment_query_reads_hits_from_published_epoch_with_duckdb_query_engine() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/attachment-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("config override: {error}"));
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = fixture_service(&temp_dir);
    let hits = vec![
        sample_hit(
            "topology.png",
            "docs/alpha.md",
            "assets/topology.png",
            "image",
        ),
        sample_hit("spec.pdf", "docs/alpha.md", "files/spec.pdf", "pdf"),
    ];
    publish_attachment_hits(&service, "fp-duckdb", &hits).await;

    let results = search_attachment_hits(&service, "topology", 5, &[], &[], false)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].attachment_name, "topology.png");
    assert!(results[0].score > 0.0);
}
