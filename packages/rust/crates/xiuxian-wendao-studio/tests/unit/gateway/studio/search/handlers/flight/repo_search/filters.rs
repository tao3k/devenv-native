use super::{
    Arc, HashSet, PathBuf, RepoSearchRequestFilters, SearchMaintenancePolicy,
    SearchManifestKeyspace, SearchPlaneService, StudioRepoSearchFlightRouteProvider,
    bootstrap_sample_repo_search_content, create_dir_all_or_panic, repo_search_batch_or_panic,
    repo_search_request, string_column, tempdir_or_panic,
};

#[tokio::test]
async fn studio_repo_search_flight_provider_applies_language_filters() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-filters"),
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
            "alpha",
            10,
            RepoSearchRequestFilters {
                language_filters: HashSet::from(["markdown".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one markdown-filtered search batch",
    )
    .await;

    let paths = string_column(&batch, "path");
    let languages = string_column(&batch, "language");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "README.md");
    assert_eq!(languages.value(0), "markdown");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_applies_path_prefix_filters() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-prefixes"),
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
            "flightbridgetoken",
            10,
            RepoSearchRequestFilters {
                path_prefixes: HashSet::from(["src/flight".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one path-filtered search batch",
    )
    .await;

    let paths = string_column(&batch, "path");

    assert_eq!(batch.num_rows(), 1);
    assert!(paths.value(0).starts_with("src/flight"));
}

#[tokio::test]
async fn studio_repo_search_flight_provider_applies_title_filters() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-titles"),
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
            "alpha",
            10,
            RepoSearchRequestFilters {
                title_filters: HashSet::from(["readme".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one title-filtered search batch",
    )
    .await;

    let paths = string_column(&batch, "path");
    let titles = string_column(&batch, "title");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "README.md");
    assert_eq!(titles.value(0), "README.md");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_applies_tag_filters() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-tags"),
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
            "alpha",
            10,
            RepoSearchRequestFilters {
                tag_filters: HashSet::from(["lang:markdown".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one tag-filtered search batch",
    )
    .await;

    let paths = string_column(&batch, "path");
    let languages = string_column(&batch, "language");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "README.md");
    assert_eq!(languages.value(0), "markdown");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_applies_filename_filters() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-filenames"),
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
            "alpha",
            10,
            RepoSearchRequestFilters {
                filename_filters: HashSet::from(["readme.md".to_string()]),
                ..RepoSearchRequestFilters::default()
            },
        ),
        "provider should materialize one filename-filtered search batch",
    )
    .await;

    let paths = string_column(&batch, "path");
    let doc_ids = string_column(&batch, "doc_id");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "README.md");
    assert_eq!(doc_ids.value(0), "README.md");
}
