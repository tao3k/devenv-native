#[cfg(feature = "duckdb")]
use serial_test::serial;
use std::fs;
use std::sync::Arc;

use crate::gateway::studio::search::handlers::code_search::build_repo_content_search_hits;
use crate::gateway::studio::search::handlers::knowledge::{
    build_intent_cache_key, build_intent_search_response, load_intent_search_response_with_metadata,
};
use crate::gateway::studio::search::handlers::queries::SearchQuery;
#[cfg(feature = "duckdb")]
use crate::gateway::studio::search::handlers::tests::write_search_duckdb_runtime_override;
#[cfg(feature = "duckdb")]
use crate::gateway::studio::search::handlers::tests::{
    configure_local_workspace, publish_knowledge_section_index, publish_local_symbol_index,
};
use crate::gateway::studio::search::handlers::tests::{
    publish_repo_content_chunk_index, publish_repo_entity_index, sample_repo_analysis,
    test_studio_state, test_studio_state_with_cache,
};
use crate::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};

#[tokio::test]
async fn build_intent_cache_key_is_stable_for_reordered_repo_config() {
    let studio = test_studio_state_with_cache();
    studio.seed_configured_owners_for_tests(
        crate::gateway::studio::types::UiConfig {
            projects: Vec::new(),
            repo_projects: vec![
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "alpha".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["julia".to_string()],
                },
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "beta".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["modelica".to_string()],
                },
            ],
        },
        false,
    );
    let left_key = build_intent_cache_key(
        &studio,
        "solve",
        "solve",
        None,
        10,
        Some("code_search"),
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("left intent cache key: {error:?}"));
    studio.seed_configured_owners_for_tests(
        crate::gateway::studio::types::UiConfig {
            projects: Vec::new(),
            repo_projects: vec![
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "beta".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["modelica".to_string()],
                },
                crate::gateway::studio::types::UiRepoProjectConfig {
                    id: "alpha".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["julia".to_string()],
                },
            ],
        },
        false,
    );
    let right_key = build_intent_cache_key(
        &studio,
        "solve",
        "solve",
        None,
        10,
        Some("code_search"),
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("right intent cache key: {error:?}"));

    assert!(
        left_key.is_some(),
        "expected left intent cache key to exist"
    );
    assert_eq!(left_key, right_key);
}

#[tokio::test]
async fn build_intent_search_response_includes_repo_content_hits_for_debug_lookup() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let valid_repo = temp.path().join("ValidPkg");
    fs::create_dir_all(valid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create valid src: {error}"));
    fs::write(
        valid_repo.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(valid_repo.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    let snapshot = Arc::new(RepoIndexSnapshot {
        repo_id: "valid".to_string(),
        analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
    });
    publish_repo_content_chunk_index(
        &studio,
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
    studio.repo_index.set_snapshot_for_test(&snapshot);
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-22T00:00:00Z".to_string()),
        attempt_count: 1,
    });
    let direct_hits = build_repo_content_search_hits(&studio, "valid", "lang:julia reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("direct repo content search hits: {error:?}"));
    assert_eq!(direct_hits.len(), 1);
    assert_eq!(direct_hits[0].path, "src/ValidPkg.jl");

    let response = build_intent_search_response(
        &studio,
        "lang:julia reexport",
        "lang:julia reexport",
        Some("valid"),
        10,
        Some("debug_lookup".to_string()),
    )
    .await
    .unwrap_or_else(|error| panic!("intent search response: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("intent_hybrid"));
    assert_eq!(response.graph_confidence_score, Some(0.0));
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("file") && hit.path == "src/ValidPkg.jl"),
        "expected repo content hit in hybrid intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn load_intent_search_response_reports_repo_content_flight_transport_metadata() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let valid_repo = temp.path().join("ValidPkg");
    fs::create_dir_all(valid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create valid src: {error}"));
    fs::write(
        valid_repo.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(valid_repo.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    let snapshot = Arc::new(RepoIndexSnapshot {
        repo_id: "valid".to_string(),
        analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
    });
    publish_repo_content_chunk_index(
        &studio,
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
    studio.repo_index.set_snapshot_for_test(&snapshot);
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-22T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let (response, metadata) = load_intent_search_response_with_metadata(
        &studio,
        SearchQuery {
            q: Some("lang:julia reexport".to_string()),
            intent: Some("debug_lookup".to_string()),
            repo: Some("valid".to_string()),
            limit: Some(10),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("intent search response with metadata: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("intent_hybrid"));
    assert_eq!(metadata.repo_content_transport, Some("flight_contract"));
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn load_intent_search_response_reports_duckdb_local_source_query_engines() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/intent-local-source-tmp"
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
        "# duckdb_focus\n\nThis note documents the duckdb_focus workflow.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    fs::write(workspace.join("src/lib.rs"), "pub fn duckdb_focus() {}\n")
        .unwrap_or_else(|error| panic!("write source file: {error}"));

    let mut studio = test_studio_state();
    configure_local_workspace(&mut studio, workspace.as_path());
    publish_knowledge_section_index(&studio).await;
    publish_local_symbol_index(&studio).await;

    let (response, metadata) = load_intent_search_response_with_metadata(
        &studio,
        SearchQuery {
            q: Some("duckdb_focus".to_string()),
            intent: Some("debug_lookup".to_string()),
            repo: None,
            limit: Some(10),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("intent search response with local duckdb metadata: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("intent_hybrid"));
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.path == "notes/duckdb_focus.md"),
        "expected knowledge hit in intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    assert!(
        response.hits.iter().any(|hit| hit.path == "src/lib.rs"),
        "expected local symbol hit in intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    assert_eq!(metadata.knowledge_query_engine, Some("duckdb"));
    assert_eq!(metadata.local_symbol_query_engine, Some("duckdb"));
    assert_eq!(metadata.repo_query_engine, None);
}

#[cfg(feature = "duckdb")]
#[tokio::test]
#[serial]
async fn load_intent_search_response_reports_duckdb_repo_query_engine() {
    let _temp = write_search_duckdb_runtime_override(
        r#"[search.duckdb]
enabled = true
database_path = ":memory:"
temp_directory = ".cache/duckdb/intent-repo-source-tmp"
threads = 2
"#,
    )
    .unwrap_or_else(|error| panic!("write duckdb runtime override: {error}"));

    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let valid_repo = temp.path().join("ValidPkg");
    fs::create_dir_all(valid_repo.join("src"))
        .unwrap_or_else(|error| panic!("create valid src: {error}"));
    fs::write(
        valid_repo.join("Project.toml"),
        "name = \"ValidPkg\"\nuuid = \"00000000-0000-0000-0000-000000000001\"\n",
    )
    .unwrap_or_else(|error| panic!("write project: {error}"));

    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(valid_repo.display().to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    let snapshot = Arc::new(RepoIndexSnapshot {
        repo_id: "valid".to_string(),
        analysis: Arc::new(crate::analyzers::RepositoryAnalysisOutput::default()),
    });
    publish_repo_content_chunk_index(
        &studio,
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
    studio.repo_index.set_snapshot_for_test(&snapshot);
    studio.repo_index.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "valid".to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("abc123".to_string()),
        updated_at: Some("2026-03-22T00:00:00Z".to_string()),
        attempt_count: 1,
    });

    let (_response, metadata) = load_intent_search_response_with_metadata(
        &studio,
        SearchQuery {
            q: Some("lang:julia reexport".to_string()),
            intent: Some("debug_lookup".to_string()),
            repo: Some("valid".to_string()),
            limit: Some(10),
        },
    )
    .await
    .unwrap_or_else(|error| panic!("intent search response with repo duckdb metadata: {error:?}"));

    assert_eq!(metadata.repo_query_engine, Some("duckdb"));
    assert_eq!(metadata.repo_content_transport, Some("flight_contract"));
}

#[tokio::test]
async fn build_intent_search_response_includes_repo_entity_hits_for_debug_lookup() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::gateway::studio::types::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::gateway::studio::types::UiRepoProjectConfig {
            id: "valid".to_string(),
            root: Some(".".to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });
    let analysis = sample_repo_analysis("valid");
    publish_repo_entity_index(&studio, "valid", &analysis).await;
    studio
        .repo_index
        .set_snapshot_for_test(&Arc::new(RepoIndexSnapshot {
            repo_id: "valid".to_string(),
            analysis: Arc::new(analysis),
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

    let response = build_intent_search_response(
        &studio,
        "reexport",
        "reexport",
        Some("valid"),
        10,
        Some("debug_lookup".to_string()),
    )
    .await
    .unwrap_or_else(|error| panic!("intent search response: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("intent_hybrid"));
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol")
                && hit.path == "src/BaseModelica.jl"),
        "expected repo entity hit in hybrid intent response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}
