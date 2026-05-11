use super::{
    Arc, PathBuf, RepoSearchFlightRouteProvider, RepoSearchRequestFilters, SearchMaintenancePolicy,
    SearchManifestKeyspace, SearchPlaneService, StudioRepoSearchFlightRouteProvider, StudioState,
    SyncMode, UiConfig, UiRepoProjectConfig, commit_all_or_panic, configured_repositories,
    create_dir_all_or_panic, init_git_repo_or_panic, repo_document, repo_search_batch_or_panic,
    repo_search_request, resolve_registered_repository_source, string_column, tempdir_or_panic,
    write_file_or_panic,
};

#[tokio::test]
async fn studio_repo_search_flight_provider_reads_repo_content_hits() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider"),
        SearchMaintenancePolicy::default(),
    ));
    let repo_id = "alpha/repo";
    let documents = vec![
        repo_document("src/lib.rs", "rust", "pub fn alpha_beta() {}\n"),
        repo_document("src/other.rs", "rust", "pub fn unrelated() {}\n"),
    ];
    service
        .publish_repo_content_chunks_with_revision(repo_id, &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("repo content publication should succeed: {error}"));

    let provider = StudioRepoSearchFlightRouteProvider::new(Arc::clone(&service));
    let batch = repo_search_batch_or_panic(
        &provider,
        &repo_search_request(repo_id, "alpha", 5, RepoSearchRequestFilters::default()),
        "provider should materialize one search batch",
    )
    .await;

    let doc_ids = string_column(&batch, "doc_id");
    let paths = string_column(&batch, "path");
    let languages = string_column(&batch, "language");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(doc_ids.value(0), "lib.rs");
    assert_eq!(paths.value(0), "src/lib.rs");
    assert_eq!(languages.value(0), "rust");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_falls_back_to_search_only_checkout_content() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    let source_root = temp_dir.path().join("lance-source");
    create_dir_all_or_panic(&project_root, "project root should build");
    create_dir_all_or_panic(&source_root, "source root should build");
    init_git_repo_or_panic(&source_root, "source repo should initialize");
    create_dir_all_or_panic(source_root.join("src"), "source src should build");
    write_file_or_panic(
        source_root.join("src/lib.rs"),
        "pub fn lance_kernel() -> &'static str { \"lance source fallback\" }\n",
        "source file should be written",
    );
    commit_all_or_panic(&source_root, "init", "source repo should commit");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-source-fallback"),
        SearchMaintenancePolicy::default(),
    ));
    let studio = Arc::new(StudioState::new());
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: Vec::new(),
        repo_projects: vec![UiRepoProjectConfig {
            id: "lance".to_string(),
            root: None,
            url: Some(source_root.display().to_string()),
            git_ref: None,
            refresh: Some("manual".to_string()),
            plugins: vec!["ast-grep".to_string()],
        }],
    });

    let provider =
        StudioRepoSearchFlightRouteProvider::with_studio(Arc::clone(&service), Arc::clone(&studio));
    let batch = repo_search_batch_or_panic(
        &provider,
        &repo_search_request("lance", "lance", 5, RepoSearchRequestFilters::default()),
        "provider should fall back to checkout-backed repo search",
    )
    .await;

    let paths = string_column(&batch, "path");
    let languages = string_column(&batch, "language");
    let match_reasons = string_column(&batch, "match_reason");

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "src/lib.rs");
    assert_eq!(languages.value(0), "rust");
    assert_eq!(match_reasons.value(0), "repo_content_search");

    let repository = configured_repositories(studio.as_ref())
        .into_iter()
        .find(|repository| repository.id == "lance")
        .unwrap_or_else(|| panic!("configured repository should exist"));
    let materialized = resolve_registered_repository_source(
        &repository,
        studio.config_root.as_path(),
        SyncMode::Status,
    )
    .unwrap_or_else(|error| panic!("materialized checkout should resolve: {error}"));
    std::fs::remove_dir_all(materialized.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    if let Some(mirror_root) = materialized.mirror_root {
        std::fs::remove_dir_all(mirror_root).ok();
    }
}

#[tokio::test]
async fn studio_repo_search_flight_provider_uses_published_hits_for_unregistered_repo() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-unregistered"),
        SearchMaintenancePolicy::default(),
    ));
    let repo_id = "docs";
    service
        .publish_repo_content_chunks_with_revision(
            repo_id,
            &[repo_document(
                "docs/search.md",
                "markdown",
                "# SearchStrategyFlow\n\nOwnership boundary validation path.\n",
            )],
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("repo content publication should succeed: {error}"));

    let provider = StudioRepoSearchFlightRouteProvider::with_studio(
        Arc::clone(&service),
        Arc::new(StudioState::new()),
    );
    let batch = repo_search_batch_or_panic(
        &provider,
        &repo_search_request(
            repo_id,
            "SearchStrategyFlow",
            5,
            RepoSearchRequestFilters::default(),
        ),
        "provider should use published repo-content hits for unregistered repos",
    )
    .await;

    let paths = string_column(&batch, "path");
    assert_eq!(batch.num_rows(), 1);
    assert_eq!(paths.value(0), "docs/search.md");
}

#[tokio::test]
async fn studio_repo_search_flight_provider_rejects_blank_repo_id() {
    let temp_dir = tempdir_or_panic("temp dir should build");
    let project_root = temp_dir.path().join("project");
    let storage_root = temp_dir.path().join("storage");
    create_dir_all_or_panic(&project_root, "project root should build");

    let service = Arc::new(SearchPlaneService::with_paths(
        PathBuf::from(&project_root),
        PathBuf::from(&storage_root),
        SearchManifestKeyspace::new("xiuxian:test:flight-repo-search-provider-blank"),
        SearchMaintenancePolicy::default(),
    ));
    let provider = StudioRepoSearchFlightRouteProvider::new(service);
    let Err(error) = provider
        .repo_search_batch(&repo_search_request(
            "   ",
            "alpha",
            5,
            RepoSearchRequestFilters::default(),
        ))
        .await
    else {
        panic!("blank repo id should fail");
    };
    assert_eq!(
        error,
        "repo-search Flight request repo_id must not be blank"
    );
}
