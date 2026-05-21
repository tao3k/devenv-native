#[cfg(feature = "duckdb")]
use serial_test::serial;
use std::fs;
use std::sync::Arc;

use crate::studio::arrow_types::LanceStringArray;
#[cfg(feature = "duckdb")]
use crate::studio::search::handlers::tests::write_search_duckdb_runtime_override;
#[cfg(feature = "duckdb")]
use crate::studio::search::handlers::tests::{
    configure_local_workspace, publish_knowledge_section_index, publish_local_symbol_index,
};
use crate::studio::search::handlers::tests::{publish_repo_content_chunk_index, test_studio_state};
use crate::transport::{SEARCH_INTENT_ROUTE, SearchFlightRouteProvider};
use xiuxian_wendao::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};

use super::StudioIntentSearchFlightRouteProvider;

#[tokio::test]
async fn studio_intent_flight_provider_reads_repo_backed_hits() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let valid_repo = temp.path().join("ValidPkg");
    fs::create_dir_all(valid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create valid src: {error}"));
    fs::write(
        valid_repo.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));

    let studio = Arc::new(test_studio_state());
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(valid_repo.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia-code-parser".to_string()],
        }],
    });
    publish_repo_content_chunk_index(
        studio.as_ref(),
        "valid",
        vec![RepoCodeDocument {
            path: "src/ValidPkg.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module ValidPkg\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 62,
            modified_unix_ms: 0,
        }],
    )
    .await;
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: "valid".to_string(),
            analysis: Arc::new(xiuxian_wendao::analyzers::RepositoryAnalysisOutput::default()),
        }));
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-22T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let provider = StudioIntentSearchFlightRouteProvider::new(Arc::clone(&studio));
    let response = provider
        .search_batch(
            SEARCH_INTENT_ROUTE,
            "lang:julia reexport",
            10,
            Some("code_search"),
            Some("valid"),
        )
        .await
        .unwrap_or_else(|error| panic!("intent-search Flight batch: {error}"));
    let batch = response.batch;

    let Some(paths) = batch
        .column_by_name("path")
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
    else {
        panic!("path should decode as Utf8");
    };
    let Some(doc_types) = batch
        .column_by_name("docType")
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
    else {
        panic!("docType should decode as Utf8");
    };

    assert!(batch.num_rows() >= 1);
    assert_eq!(paths.value(0), "src/ValidPkg.jl");
    assert_eq!(doc_types.value(0), "file");
}

#[tokio::test]
async fn studio_intent_flight_provider_rejects_non_intent_routes() {
    let provider = StudioIntentSearchFlightRouteProvider::new(Arc::new(test_studio_state()));

    let Err(error) = provider
        .search_batch("/search/symbols", "anything", 5, None, None)
        .await
    else {
        panic!("non-intent route should fail");
    };

    assert_eq!(
        error,
        "studio intent Flight provider only supports route `/search/intent`, got `/search/symbols`"
    );
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn studio_intent_flight_provider_reads_local_duckdb_fed_hits() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/intent-flight-local-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("write duckdb runtime override: {error}"));

    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let workspace = temp.path().join("workspace");
    fs::create_dir_all(workspace.join("notes"))
        .unwrap_or_else(|error| panic!("create notes dir: {error}"));
    fs::create_dir_all(workspace.join("src"))
        .unwrap_or_else(|error| panic!("create src dir: {error}"));
    fs::write(
        workspace.join("notes/duckdb_focus.md"),
        "# duckdb_focus\n\nDuckDB intent note.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    fs::write(workspace.join("src/lib.rs"), "pub fn duckdb_focus() {}\n")
        .unwrap_or_else(|error| panic!("write source file: {error}"));

    let mut studio = test_studio_state();
    configure_local_workspace(&mut studio, workspace.as_path());
    publish_knowledge_section_index(&studio).await;
    publish_local_symbol_index(&studio).await;

    let provider = StudioIntentSearchFlightRouteProvider::new(Arc::new(studio));
    let response = provider
        .search_batch(
            SEARCH_INTENT_ROUTE,
            "duckdb_focus",
            10,
            Some("debug_lookup"),
            None,
        )
        .await
        .unwrap_or_else(|error| panic!("intent-search Flight batch: {error}"));
    let batch = response.batch;

    let Some(paths) = batch
        .column_by_name("path")
        .and_then(|column| column.as_any().downcast_ref::<LanceStringArray>())
    else {
        panic!("path should decode as Utf8");
    };

    let values = (0..batch.num_rows())
        .map(|index| paths.value(index))
        .collect::<Vec<_>>();
    assert!(values.contains(&"notes/duckdb_focus.md"));
    assert!(values.contains(&"src/lib.rs"));
}
