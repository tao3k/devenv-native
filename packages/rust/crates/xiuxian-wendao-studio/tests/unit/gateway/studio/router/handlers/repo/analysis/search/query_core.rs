use super::support::sample_repo_entity_service;
use super::{
    PathBuf, SearchMaintenancePolicy, SearchPlaneService, StudioApiError,
    assert_studio_json_snapshot, query_repo_entity_example_results_if_published,
    query_repo_entity_import_results_if_published, query_repo_entity_module_results_if_published,
    query_repo_entity_symbol_results_if_published,
};

#[tokio::test]
async fn repo_entity_query_core_returns_none_when_publication_is_not_ready() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let service = SearchPlaneService::with_test_cache(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        super::support::unique_repo_gateway_keyspace("entity-module-not-ready", temp_dir.path()),
        SearchMaintenancePolicy::default(),
    );

    let result = query_repo_entity_module_results_if_published(
        &service,
        "alpha/repo",
        "BaseModelica",
        5,
        false,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return none: {error:?}"));

    assert!(result.is_none());
}

#[tokio::test]
async fn repo_entity_query_core_module_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_module_payload").await;

    let result = query_repo_entity_module_results_if_published(
        &service,
        "alpha/repo",
        "BaseModelica",
        5,
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return module payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_module_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_symbol_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_symbol_payload").await;

    let result =
        query_repo_entity_symbol_results_if_published(&service, "alpha/repo", "solve", 5, true)
            .await
            .unwrap_or_else(|error| panic!("query helper should return symbol payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_symbol_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_example_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_example_payload").await;

    let result =
        query_repo_entity_example_results_if_published(&service, "alpha/repo", "solve", 5, true)
            .await
            .unwrap_or_else(|error| {
                panic!("query helper should return example payload: {error:?}")
            });

    assert_studio_json_snapshot("repo_analysis_example_search_plane_payload", result);
}

#[tokio::test]
async fn repo_entity_query_core_import_payload_snapshot() {
    let (_temp_dir, service) =
        sample_repo_entity_service("xiuxian:test:repo_entity_import_payload").await;

    let result = query_repo_entity_import_results_if_published(
        &service,
        "alpha/repo",
        Some("SciMLBase".to_string()),
        Some("BaseModelica".to_string()),
        5,
        true,
    )
    .await
    .unwrap_or_else(|error| panic!("query helper should return import payload: {error:?}"));

    assert_studio_json_snapshot("repo_analysis_import_query_core_payload", result);
}

#[test]
fn repo_entity_query_core_error_mapping_preserves_gateway_contract() {
    let error = StudioApiError::internal(
        "REPO_MODULE_SEARCH_FAILED",
        "Repo module search task failed",
        Some("broken repo entity payload".to_string()),
    );

    assert_eq!(error.code(), "REPO_MODULE_SEARCH_FAILED");
    assert_eq!(
        error.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}
