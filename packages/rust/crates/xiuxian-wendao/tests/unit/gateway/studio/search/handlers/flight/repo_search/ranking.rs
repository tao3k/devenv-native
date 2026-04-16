use super::*;

#[tokio::test]
async fn studio_repo_search_flight_provider_exposes_exact_match_tag() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-exact-tag"),
        SearchMaintenancePolicy::default(),
    ));
    bootstrap_sample_repo_search_content(service.as_ref(), "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));

    let provider = StudioRepoSearchFlightRouteProvider::new(Arc::clone(&service));
    let batch = repo_search_batch_or_panic(
        &provider,
        &repo_search_request(
            "alpha/repo",
            "searchonlytoken",
            10,
            RepoSearchRequestFilters {
                tag_filters: HashSet::from(["match:exact".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one exact-match-tagged search batch",
    )
    .await;

    let paths = string_column(&batch, "path");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "src/search.rs");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_prefers_exact_case_match_over_folded_match() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-exact-rank"),
        SearchMaintenancePolicy::default(),
    ));
    bootstrap_sample_repo_search_content(service.as_ref(), "alpha/repo")
        .await
        .unwrap_or_else(|error| panic!("sample bootstrap should publish repo content: {error}"));

    let provider = StudioRepoSearchFlightRouteProvider::new(Arc::clone(&service));
    let batch = repo_search_batch_or_panic(
        &provider,
        &repo_search_request(
            "alpha/repo",
            "CamelBridgeToken",
            2,
            RepoSearchRequestFilters::default(),
        ),
        "provider should materialize one exact-ranked search batch",
    )
    .await;

    let paths = string_column(&batch, "path");
    let scores = float_column(&batch, "score");

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(paths.value(0), "docs/CamelBridge.md");
    assert_eq!(paths.value(1), "src/camelbridge.rs");
    assert!(scores.value(0) > scores.value(1));
}
