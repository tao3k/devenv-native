use super::support::*;

#[tokio::test]
async fn run_repository_analysis_returns_empty_analysis_for_search_only_repositories() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let coordinator = new_coordinator(SearchPlaneService::new(PathBuf::from(".")));
    let repository = RegisteredRepository {
        id: "search-only-analysis".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
    };

    let analysis = coordinator
        .run_repository_analysis(repository)
        .await
        .unwrap_or_else(|error| panic!("search-only analysis fallback: {error}"));

    assert!(analysis.repository.is_none());
    assert!(analysis.modules.is_empty());
    assert!(analysis.symbols.is_empty());
    assert!(analysis.examples.is_empty());
    assert!(analysis.docs.is_empty());
    assert!(analysis.imports.is_empty());
    assert!(analysis.relations.is_empty());
    assert!(analysis.diagnostics.is_empty());
}

#[tokio::test]
async fn search_only_repository_task_publishes_repo_backed_corpora_for_supported_code_documents() {
    let tempdir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(tempdir.path());
    fs::create_dir_all(tempdir.path().join("src"))
        .unwrap_or_else(|error| panic!("create src: {error}"));
    fs::write(
        tempdir.path().join("src/Alpha.jl"),
        "function alpha()\nend\n",
    )
    .unwrap_or_else(|error| panic!("write Julia source: {error}"));
    commit_all(tempdir.path(), "initial search-only repo");
    let revision = discover_checkout_metadata(tempdir.path())
        .and_then(|metadata| metadata.revision)
        .unwrap_or_else(|| panic!("discover revision"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        tempdir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-search-only-publication"),
        SearchMaintenancePolicy::default(),
    );
    let coordinator = Arc::new(new_coordinator(search_plane.clone()));
    coordinator.start();

    let repository = RegisteredRepository {
        id: "search-only-runtime".to_string(),
        path: Some(tempdir.path().to_path_buf()),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("ast-grep".to_string())],
    };

    let enqueued = coordinator.sync_repositories(vec![repository]);
    assert_eq!(enqueued, vec!["search-only-runtime".to_string()]);

    let status = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let status = coordinator.status_response(Some("search-only-runtime"));
            if let Some(repo) = status.repos.first()
                && matches!(
                    repo.phase,
                    crate::repo_index::types::RepoIndexPhase::Ready
                        | crate::repo_index::types::RepoIndexPhase::Failed
                        | crate::repo_index::types::RepoIndexPhase::Unsupported
                )
            {
                return status;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap_or_else(|error| panic!("search-only repo should reach terminal state: {error}"));
    assert_eq!(status.total, 1);
    assert_eq!(
        status.repos[0].phase,
        crate::repo_index::types::RepoIndexPhase::Ready,
        "search-only repo should become ready instead of failing: {:?}",
        status.repos
    );

    let publication_state = search_plane
        .repo_search_publication_state("search-only-runtime")
        .await;
    assert_eq!(
        publication_state.availability,
        RepoSearchAvailability::Searchable
    );

    let entity_publication = search_plane
        .repo_publication_for_revision(
            SearchCorpusKind::RepoEntity,
            "search-only-runtime",
            revision.as_str(),
        )
        .await
        .unwrap_or_else(|| panic!("repo-entity publication for search-only repo"));
    let content_publication = search_plane
        .repo_publication_for_revision(
            SearchCorpusKind::RepoContentChunk,
            "search-only-runtime",
            revision.as_str(),
        )
        .await
        .unwrap_or_else(|| panic!("repo-content publication for search-only repo"));
    assert_eq!(entity_publication.row_count, 0);
    assert_eq!(content_publication.row_count, 2);

    let hits = search_plane
        .search_repo_content_chunks("search-only-runtime", "alpha", &HashSet::new(), 10)
        .await
        .unwrap_or_else(|error| panic!("search search-only repo contents: {error}"));
    assert!(
        hits.iter().any(|hit| hit.path == "src/Alpha.jl"),
        "expected search-only repo content hit to include src/Alpha.jl: {hits:?}"
    );

    coordinator.stop();
}

#[tokio::test]
async fn refresh_status_snapshot_synchronizes_search_plane_runtime() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-runtime-sync"),
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane.clone());
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "pending".to_string(),
        phase: crate::repo_index::types::RepoIndexPhase::Queued,
        queue_position: None,
        last_error: None,
        last_revision: None,
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "skipped".to_string(),
        phase: crate::repo_index::types::RepoIndexPhase::Failed,
        queue_position: None,
        last_error: Some("boom".to_string()),
        last_revision: None,
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });

    let pending = search_plane.repo_search_publication_state("pending").await;
    let skipped = search_plane.repo_search_publication_state("skipped").await;

    assert_eq!(pending.availability, RepoSearchAvailability::Pending);
    assert_eq!(skipped.availability, RepoSearchAvailability::Skipped);
}

#[tokio::test]
async fn stop_releases_background_runner_arc() {
    let coordinator = Arc::new(new_coordinator(SearchPlaneService::new(PathBuf::from("."))));
    let weak = Arc::downgrade(&coordinator);

    coordinator.start();
    tokio::task::yield_now().await;
    coordinator.stop();
    drop(coordinator);

    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            if weak.upgrade().is_none() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap_or_else(|error| panic!("runner arc should be released after stop: {error}"));
}

#[tokio::test]
async fn refresh_status_snapshot_synchronizes_repo_backed_corpus_statuses() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        temp_dir.path().join("search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:repo-status-sync"),
        SearchMaintenancePolicy::default(),
    );
    let documents = vec![crate::repo_index::types::RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    search_plane
        .publish_repo_entities_with_revision(
            "alpha/repo",
            &RepositoryAnalysisOutput {
                modules: vec![crate::analyzers::ModuleRecord {
                    repo_id: "alpha/repo".to_string(),
                    module_id: "module:alpha".to_string(),
                    qualified_name: "Alpha".to_string(),
                    path: "src/lib.rs".to_string(),
                }],
                ..RepositoryAnalysisOutput::default()
            },
            &documents,
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision("alpha/repo", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));
    let coordinator = new_coordinator(search_plane.clone());
    coordinator.set_status_for_test(RepoIndexEntryStatus {
        repo_id: "alpha/repo".to_string(),
        phase: crate::repo_index::types::RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some("rev-1".to_string()),
        updated_at: Some(timestamp_now()),
        attempt_count: 1,
    });

    tokio::time::timeout(Duration::from_secs(1), async {
        loop {
            let snapshot = search_plane.status();
            let Some(repo_entity) = snapshot
                .corpora
                .iter()
                .find(|entry| entry.corpus == SearchCorpusKind::RepoEntity)
            else {
                panic!("repo entity row");
            };
            let Some(repo_content) = snapshot
                .corpora
                .iter()
                .find(|entry| entry.corpus == SearchCorpusKind::RepoContentChunk)
            else {
                panic!("repo content row");
            };
            if repo_entity.phase == SearchPlanePhase::Ready
                && repo_content.phase == SearchPlanePhase::Ready
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap_or_else(|error| panic!("repo-backed corpus status should synchronize: {error}"));
}
