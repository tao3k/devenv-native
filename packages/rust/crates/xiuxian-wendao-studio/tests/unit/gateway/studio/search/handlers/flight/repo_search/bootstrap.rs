use super::{
    HashSet, PathBuf, SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneService,
    bootstrap_sample_repo_search_content, create_dir_all_or_panic, tempdir_or_panic,
};

#[tokio::test]
async fn bootstrap_sample_repo_search_content_publishes_queryable_rows() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-bootstrap"),
        SearchMaintenancePolicy::default(),
    );
    bootstrap_sample_repo_search_content(&service, "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));

    let hits = service
        .search_repo_content_chunks("alpha/repo", "flight", &HashSet::new(), 5)
        .await
        .unwrap_or_else(|error| panic!("bootstrapped repo should be searchable: {error}"));

    assert!(!hits.is_empty());
    assert!(hits.iter().any(|hit| hit.path == "src/flight.rs"));
    assert!(hits.iter().any(|hit| hit.path == "src/flight_search.rs"));
}

#[tokio::test]
async fn bootstrap_sample_repo_search_content_respects_query_and_limit() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-bootstrap-query-limit"),
        SearchMaintenancePolicy::default(),
    );
    bootstrap_sample_repo_search_content(&service, "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));

    let search_hits = service
        .search_repo_content_chunks("alpha/repo", "searchonlytoken", &HashSet::new(), 1)
        .await
        .unwrap_or_else(|error| {
            panic!("bootstrapped repo should be searchable by search keyword: {error}")
        });
    let flight_hits = service
        .search_repo_content_chunks("alpha/repo", "flightbridgetoken", &HashSet::new(), 5)
        .await
        .unwrap_or_else(|error| {
            panic!("bootstrapped repo should be searchable by combined keywords: {error}")
        });

    assert_eq!(search_hits.len(), 1);
    assert_eq!(search_hits[0].path, "src/search.rs");
    assert!(
        flight_hits
            .iter()
            .any(|hit| hit.path == "src/flight_search.rs")
    );
}

#[tokio::test]
async fn bootstrap_sample_repo_search_content_uses_path_order_for_exact_match_ties() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-bootstrap-rank-tie"),
        SearchMaintenancePolicy::default(),
    );
    bootstrap_sample_repo_search_content(&service, "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));

    let hits = service
        .search_repo_content_chunks("alpha/repo", "ranktieexacttoken", &HashSet::new(), 1)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "bootstrapped repo should expose deterministic exact-match tie ordering: {error}"
            )
        });

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "src/a_rank.rs");
}

#[tokio::test]
async fn bootstrap_sample_repo_search_content_persists_across_service_restart() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let writer = SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-bootstrap-persist"),
        SearchMaintenancePolicy::default(),
    );
    bootstrap_sample_repo_search_content(&writer, "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));
    drop(writer);

    let reader = SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-bootstrap-persist"),
        SearchMaintenancePolicy::default(),
    );
    let hits = reader
        .search_repo_content_chunks("alpha/repo", "alpha", &HashSet::new(), 5)
        .await
        .unwrap_or_else(|error| {
            panic!("bootstrapped repo should remain searchable after restart: {error}")
        });

    assert!(!hits.is_empty());
    assert!(hits.iter().any(|hit| hit.path == "src/lib.rs"));
}
