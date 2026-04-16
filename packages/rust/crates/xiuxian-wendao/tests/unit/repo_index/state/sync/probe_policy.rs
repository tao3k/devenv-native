use super::support::*;

#[tokio::test]
async fn sync_repositories_warm_starts_stale_fetch_policy_remote_when_recent_probe_matches() {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_repo = temp_dir.path().join("managed-remote-source");
    fs::create_dir_all(&source_repo).unwrap_or_else(|error| panic!("create source repo: {error}"));
    init_test_repository(&source_repo);

    let repo_id = format!("managed-remote-probe-{}", Uuid::new_v4());
    let repository = remote_repo(&repo_id, source_repo.display().to_string().as_str());
    let first =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve first ensure: {error}"));
    let second =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve second ensure: {error}"));
    let revision = discover_checkout_metadata(&second.checkout_root)
        .unwrap_or_else(|| panic!("discover checkout metadata for `{repo_id}`"))
        .revision
        .unwrap_or_else(|| panic!("missing revision for `{repo_id}`"));

    set_mirror_fetch_age(
        second
            .mirror_root
            .as_deref()
            .unwrap_or_else(|| panic!("missing mirror root for `{repo_id}`")),
        Duration::from_secs(3 * 24 * 3600),
    );

    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace = SearchManifestKeyspace::new(format!(
        "xiuxian:test:repo-warm-start-managed-probe-{repo_id}"
    ));
    let initial_search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    initial_search_plane
        .publish_repo_entities_with_revision(
            repo_id.as_str(),
            &repo_analysis_output(repo_id.as_str()),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    initial_search_plane
        .publish_repo_content_chunks_with_revision(
            repo_id.as_str(),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![repository]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some(repo_id.as_str()));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(
        status.repos[0].last_revision.as_deref(),
        Some(revision.as_str())
    );

    let Some(mirror_root) = second.mirror_root.as_ref() else {
        panic!("mirror root");
    };
    fs::remove_dir_all(mirror_root)
        .unwrap_or_else(|error| panic!("cleanup managed mirror: {error}"));
    fs::remove_dir_all(&second.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    fs::remove_dir_all(first.checkout_root).ok();
}

#[tokio::test]
async fn sync_repositories_warm_starts_stale_fetch_policy_remote_when_recent_retryable_probe_failure_exists()
 {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_repo = temp_dir.path().join("managed-remote-source");
    fs::create_dir_all(&source_repo).unwrap_or_else(|error| panic!("create source repo: {error}"));
    init_test_repository(&source_repo);

    let repo_id = format!("managed-remote-probe-failure-{}", Uuid::new_v4());
    let repository = remote_repo(&repo_id, source_repo.display().to_string().as_str());
    let first =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve first ensure: {error}"));
    let second =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve second ensure: {error}"));
    let mirror_root = second
        .mirror_root
        .as_deref()
        .unwrap_or_else(|| panic!("missing mirror root for `{repo_id}`"));
    let revision = discover_checkout_metadata(&second.checkout_root)
        .unwrap_or_else(|| panic!("discover checkout metadata for `{repo_id}`"))
        .revision
        .unwrap_or_else(|| panic!("missing revision for `{repo_id}`"));

    set_mirror_fetch_age(mirror_root, Duration::from_secs(3 * 24 * 3600));
    record_managed_remote_probe_failure(mirror_root, "operation timed out", true)
        .unwrap_or_else(|error| panic!("record retryable probe failure: {error}"));

    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace = SearchManifestKeyspace::new(format!(
        "xiuxian:test:repo-warm-start-managed-probe-failure-{repo_id}"
    ));
    let initial_search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    initial_search_plane
        .publish_repo_entities_with_revision(
            repo_id.as_str(),
            &repo_analysis_output(repo_id.as_str()),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    initial_search_plane
        .publish_repo_content_chunks_with_revision(
            repo_id.as_str(),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![repository]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some(repo_id.as_str()));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(
        status.repos[0].last_revision.as_deref(),
        Some(revision.as_str())
    );

    let Some(mirror_root) = second.mirror_root.as_ref() else {
        panic!("mirror root");
    };
    fs::remove_dir_all(mirror_root)
        .unwrap_or_else(|error| panic!("cleanup managed mirror: {error}"));
    fs::remove_dir_all(&second.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    fs::remove_dir_all(first.checkout_root).ok();
}

#[tokio::test]
async fn sync_repositories_warm_starts_stale_fetch_policy_remote_when_retryable_probe_failure_preserves_aging_success_proof()
 {
    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let source_repo = temp_dir.path().join("managed-remote-source");
    fs::create_dir_all(&source_repo).unwrap_or_else(|error| panic!("create source repo: {error}"));
    init_test_repository(&source_repo);

    let repo_id = format!("managed-remote-probe-history-{}", Uuid::new_v4());
    let repository = remote_repo(&repo_id, source_repo.display().to_string().as_str());
    let first =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve first ensure: {error}"));
    let second =
        resolve_registered_repository_source(&repository, temp_dir.path(), SyncMode::Ensure)
            .unwrap_or_else(|error| panic!("resolve second ensure: {error}"));
    let mirror_root = second
        .mirror_root
        .as_deref()
        .unwrap_or_else(|| panic!("missing mirror root for `{repo_id}`"));
    let revision = discover_checkout_metadata(&second.checkout_root)
        .unwrap_or_else(|| panic!("discover checkout metadata for `{repo_id}`"))
        .revision
        .unwrap_or_else(|| panic!("missing revision for `{repo_id}`"));

    set_mirror_fetch_age(mirror_root, Duration::from_secs(3 * 24 * 3600));
    record_managed_remote_probe_state(mirror_root, Some(revision.as_str()))
        .unwrap_or_else(|error| panic!("record probe success: {error}"));
    record_managed_remote_probe_failure(mirror_root, "operation timed out", true)
        .unwrap_or_else(|error| panic!("record retryable probe failure: {error}"));
    set_managed_remote_probe_state_age(
        mirror_root,
        Duration::from_secs(2 * 3600),
        Some(Duration::from_secs(2 * 3600)),
    );

    let storage_root = temp_dir.path().join("search-plane");
    let manifest_keyspace = SearchManifestKeyspace::new(format!(
        "xiuxian:test:repo-warm-start-managed-probe-history-{repo_id}"
    ));
    let initial_search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root.clone(),
        manifest_keyspace.clone(),
        SearchMaintenancePolicy::default(),
    );
    let documents = repo_documents();
    initial_search_plane
        .publish_repo_entities_with_revision(
            repo_id.as_str(),
            &repo_analysis_output(repo_id.as_str()),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo entities: {error}"));
    initial_search_plane
        .publish_repo_content_chunks_with_revision(
            repo_id.as_str(),
            &documents,
            Some(revision.as_str()),
        )
        .await
        .unwrap_or_else(|error| panic!("publish repo content chunks: {error}"));

    let search_plane = SearchPlaneService::with_paths(
        PathBuf::from("."),
        storage_root,
        manifest_keyspace,
        SearchMaintenancePolicy::default(),
    );
    let coordinator = new_coordinator(search_plane);

    let enqueued = coordinator.sync_repositories(vec![repository]);

    assert!(enqueued.is_empty());
    assert!(coordinator.pending_repo_ids_for_test().is_empty());

    let status = coordinator.status_response(Some(repo_id.as_str()));
    assert_eq!(status.total, 1);
    assert_eq!(status.ready, 1);
    assert_eq!(status.repos[0].phase, RepoIndexPhase::Ready);
    assert_eq!(
        status.repos[0].last_revision.as_deref(),
        Some(revision.as_str())
    );

    let Some(mirror_root) = second.mirror_root.as_ref() else {
        panic!("mirror root");
    };
    fs::remove_dir_all(mirror_root)
        .unwrap_or_else(|error| panic!("cleanup managed mirror: {error}"));
    fs::remove_dir_all(&second.checkout_root)
        .unwrap_or_else(|error| panic!("cleanup managed checkout: {error}"));
    fs::remove_dir_all(first.checkout_root).ok();
}
