use std::sync::Arc;

use crate::studio::search::handlers::code_search::{
    build_code_search_response, build_repo_entity_search_hits,
};
use crate::studio::search::handlers::tests::{
    publish_repo_content_chunk_index, publish_repo_entity_index, sample_repo_analysis,
    test_studio_state,
};
use xiuxian_wendao::repo_index::{
    RepoCodeDocument, RepoIndexEntryStatus, RepoIndexPhase, RepoIndexSnapshot,
};
use xiuxian_wendao::search::{SearchCorpusKind, SearchQueryTelemetrySource};

#[tokio::test]
async fn build_code_search_response_returns_repo_entity_hits_from_search_plane() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
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

    let direct_hits = build_repo_entity_search_hits(&studio, "valid", "reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("direct repo entity search hits: {error:?}"));
    assert!(
        direct_hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol")
                && hit.path == "src/BaseModelica.jl"),
        "expected direct repo entity symbol hit: {:?}",
        direct_hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );

    let response = build_code_search_response(&studio, "reexport".to_string(), Some("valid"), 10)
        .await
        .unwrap_or_else(|error| panic!("code search response: {error:?}"));

    assert_eq!(response.selected_mode.as_deref(), Some("code_search"));
    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol")
                && hit.path == "src/BaseModelica.jl"),
        "expected repo entity hit in code search response: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn build_code_search_response_prefers_repo_entity_hits_before_repo_content_fallback() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
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
    publish_repo_content_chunk_index(
        &studio,
        "valid",
        vec![RepoCodeDocument {
            path: "src/BaseModelica.jl".to_string(),
            language: Some("julia".to_string()),
            contents: Arc::<str>::from(
                "module BaseModelica\nusing Reexport\n@reexport using ModelingToolkit\nend\n",
            ),
            size_bytes: 67,
            modified_unix_ms: 0,
        }],
    )
    .await;
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

    let response = build_code_search_response(&studio, "reexport".to_string(), None, 10)
        .await
        .unwrap_or_else(|error| panic!("repo-wide code search response: {error:?}"));

    assert!(
        response
            .hits
            .iter()
            .any(|hit| hit.doc_type.as_deref() == Some("symbol")
                && hit.path == "src/BaseModelica.jl"),
        "expected repo entity symbol hit in repo-wide search: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
    assert!(
        response
            .hits
            .iter()
            .all(|hit| hit.doc_type.as_deref() != Some("file")),
        "repo entity hit should suppress repo-content fallback for the same repo: {:?}",
        response
            .hits
            .iter()
            .map(|hit| (&hit.path, &hit.doc_type))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn repo_entity_search_hits_record_query_core_telemetry_into_search_plane_status() {
    let studio = test_studio_state();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: Vec::new(),
        repo_projects: vec![crate::contracts::UiRepoProjectConfig {
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

    let hits = build_repo_entity_search_hits(&studio, "valid", "reexport", 10)
        .await
        .unwrap_or_else(|error| panic!("direct repo entity search hits: {error:?}"));
    assert!(
        !hits.is_empty(),
        "repo entity query-core search should return at least one hit"
    );

    let snapshot = studio.search_plane.status();
    let repo_entity = snapshot
        .corpora
        .iter()
        .find(|status| status.corpus == SearchCorpusKind::RepoEntity)
        .unwrap_or_else(|| panic!("repo entity status should be present"));
    let telemetry = repo_entity
        .last_query_telemetry
        .as_ref()
        .unwrap_or_else(|| panic!("repo entity telemetry should be present"));

    assert_eq!(telemetry.scope.as_deref(), Some("valid"));
    assert_eq!(telemetry.source, SearchQueryTelemetrySource::Scan);
    assert_eq!(
        telemetry.result_count,
        u64::try_from(hits.len()).unwrap_or(u64::MAX)
    );
    assert_eq!(
        telemetry.matched_rows,
        u64::try_from(hits.len()).unwrap_or(u64::MAX)
    );
}
