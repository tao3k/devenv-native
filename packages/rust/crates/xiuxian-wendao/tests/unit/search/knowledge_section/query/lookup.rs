//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
#[cfg(feature = "duckdb")]
use serial_test::serial;

use crate::search::knowledge_section::search_knowledge_sections;

#[cfg(feature = "duckdb")]
use super::fixtures::write_search_duckdb_runtime_override;
use super::fixtures::{fixture_service, publish_knowledge_notes};

#[tokio::test]
async fn knowledge_query_reads_hits_from_published_epoch() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let fixture = fixture_service(&temp_dir);
    publish_knowledge_notes(
        &fixture,
        "fp-1",
        &[
            (
                "notes/alpha.md",
                "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
            ),
            (
                "notes/gamma.md",
                "# Gamma\n\nGamma body.\n\n## Overview\n\nGamma section.\n",
            ),
        ],
    )
    .await;

    let results = search_knowledge_sections(&fixture.service, "Gamma body", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].stem, "gamma");
    assert_eq!(results[0].path, "notes/gamma.md");
    assert!(results[0].score > 0.0);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn knowledge_query_reads_hits_from_published_epoch_with_duckdb_query_engine() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/knowledge-query-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("config override: {error}"));
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let fixture = fixture_service(&temp_dir);
    publish_knowledge_notes(
        &fixture,
        "fp-duckdb",
        &[
            (
                "notes/alpha.md",
                "# Alpha\n\nAlpha body.\n\n## Overview\n\nAlpha section.\n",
            ),
            (
                "notes/gamma.md",
                "# Gamma\n\nGamma body.\n\n## Overview\n\nGamma section.\n",
            ),
        ],
    )
    .await;

    let results = search_knowledge_sections(&fixture.service, "Gamma body", 5)
        .await
        .unwrap_or_else(|error| panic!("query should succeed: {error}"));

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].stem, "gamma");
    assert_eq!(results[0].path, "notes/gamma.md");
    assert!(results[0].score > 0.0);
}
