use super::support::{
    PathBuf, RepoIndexPhase, SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneService, fs, new_coordinator, remote_repo, repo, repo_analysis_output,
    repo_documents,
};

#[tokio::test]
async fn sync_repositories_warm_starts_local_checkout_from_persisted_repo_publications() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace = SearchManifestKeyspace::new("xiuxian:test:repo-warm-start-local");
    let initial_search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    initial_search_plane
        .publish_repo_entities_with_revision(
            "local-repo",
            &repo_analysis_output("local-repo"),
            &documents,
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    initial_search_plane
        .publish_repo_content_chunks_with_revision("local-repo", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![repo("local-repo", "./local-repo")]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some("local-repo"));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(status.repos[0].last_revision.as_deref(), Some("rev-1"));
}

#[tokio::test]
async fn managed_remote_with_missing_assets_warm_starts_from_readable_publications() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace = SearchManifestKeyspace::new("xiuxian:test:repo-warm-start-remote");
    let initial_search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    initial_search_plane
        .publish_repo_entities_with_revision(
            "managed-remote",
            &repo_analysis_output("managed-remote"),
            &documents,
            Some("rev-1"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    initial_search_plane
        .publish_repo_content_chunks_with_revision("managed-remote", &documents, Some("rev-1"))
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![remote_repo(
        "managed-remote",
        "https://example.com/managed-remote.git",
    )]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some("managed-remote"));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(status.repos[0].last_revision.as_deref(), Some("rev-1"));
}

#[tokio::test]
async fn sync_repositories_warm_starts_from_valkey_repo_publications_after_memory_and_local_snapshot_miss()
 {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace =
        SearchManifestKeyspace::new("xiuxian:test:repo-warm-start-local-cache-only");
    let search_plane = SearchPlaneService::with_test_cache(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    search_plane
        .publish_repo_entities_with_revision(
            "cache-only-local-repo",
            &repo_analysis_output("cache-only-local-repo"),
            &documents,
            Some("rev-cache"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    search_plane
        .publish_repo_content_chunks_with_revision(
            "cache-only-local-repo",
            &documents,
            Some("rev-cache"),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    search_plane.clear_all_in_memory_repo_corpus_records_for_test();
    for corpus in [
        SearchCorpusKind::RepoEntity,
        SearchCorpusKind::RepoContentChunk,
    ] {
        fs::remove_file(search_plane.repo_corpus_record_json_path(corpus, "cache-only-local-repo"))
            .ok();
    }
    fs::remove_file(search_plane.repo_corpus_snapshot_json_path()).ok();

    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![repo(
        "cache-only-local-repo",
        "./cache-only-local-repo",
    )]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some("cache-only-local-repo"));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(status.repos[0].last_revision.as_deref(), Some("rev-cache"));
}
