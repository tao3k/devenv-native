use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::repo_index::{RepoCodeDocument, RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::cache::SearchPlaneCache;
use crate::search::service::tests::support::{
    corpus_status, ok_or_panic, publish_repo_bundle, repo_phase, repo_status_entry,
    service_test_manifest_keyspace, some_or_panic, temp_dir, unique_test_manifest_keyspace,
};
use crate::search::{
    SearchCorpusKind, SearchMaintenancePolicy, SearchPlanePhase, SearchPlaneService,
    SearchRepoRuntimeRecord,
};

fn ready_repo_status_rows(repo_ids: &[&str]) -> RepoIndexStatusResponse {
    RepoIndexStatusResponse {
        total: repo_ids.len(),
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: repo_ids.len(),
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: repo_ids
            .iter()
            .map(|repo_id| repo_status_entry(repo_id, RepoIndexPhase::Ready))
            .collect(),
    }
}

#[tokio::test]
async fn status_with_repo_runtime_hydrates_repo_corpus_status_from_repo_record_inventory() {
    let temp_dir = temp_dir();
    let keyspace = unique_test_manifest_keyspace("status-hydrate");
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );
    let documents = vec![RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;
    service.synchronize_repo_runtime(&RepoIndexStatusResponse {
        total: 1,
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 1,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: vec![repo_status_entry("alpha/repo", RepoIndexPhase::Ready)],
    });
    assert!(!service.repo_corpus_snapshot_json_path().exists());
    service.clear_all_in_memory_repo_runtime_for_test();
    fs::remove_file(service.repo_corpus_snapshot_json_path()).ok();

    let snapshot = ok_or_panic(
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let snapshot = service.status_with_repo_runtime().await;
                let repo_entity =
                    corpus_status(&snapshot, SearchCorpusKind::RepoEntity, "repo entity row");
                let repo_content = corpus_status(
                    &snapshot,
                    SearchCorpusKind::RepoContentChunk,
                    "repo content row",
                );
                if repo_entity.phase == SearchPlanePhase::Ready
                    && repo_content.phase == SearchPlanePhase::Ready
                {
                    break snapshot;
                }
                tokio::task::yield_now().await;
            }
        })
        .await,
        "repo-corpus record inventory should hydrate",
    );
    let repo_entity = corpus_status(&snapshot, SearchCorpusKind::RepoEntity, "repo entity row");
    let repo_content = corpus_status(
        &snapshot,
        SearchCorpusKind::RepoContentChunk,
        "repo content row",
    );

    assert_eq!(repo_entity.phase, SearchPlanePhase::Ready);
    assert!(repo_entity.active_epoch.is_some());
    assert!(repo_entity.row_count.unwrap_or_default() > 0);
    assert_eq!(repo_content.phase, SearchPlanePhase::Ready);
    assert!(repo_content.active_epoch.is_some());
    assert!(repo_content.row_count.unwrap_or_default() > 0);
}

#[tokio::test]
async fn synchronize_repo_runtime_removes_local_repo_record_files_for_removed_repo_ids() {
    let temp_dir = temp_dir();
    let keyspace = unique_test_manifest_keyspace("runtime-removed-local-records");
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );
    let documents = vec![RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;
    service
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&["alpha/repo"]))
        .await;

    let entity_record_path =
        service.repo_corpus_record_json_path(SearchCorpusKind::RepoEntity, "alpha/repo");
    let content_record_path =
        service.repo_corpus_record_json_path(SearchCorpusKind::RepoContentChunk, "alpha/repo");
    assert!(entity_record_path.exists());
    assert!(content_record_path.exists());

    service
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&[]))
        .await;

    assert!(!entity_record_path.exists());
    assert!(!content_record_path.exists());
}

#[tokio::test]
async fn status_with_repo_runtime_is_stable_for_reordered_ready_published_repo_rows() {
    let temp_dir = temp_dir();
    let keyspace = unique_test_manifest_keyspace("status-row-order");
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );
    let documents = vec![RepoCodeDocument {
        path: "src/lib.rs".to_string(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\n"),
        size_bytes: 14,
        modified_unix_ms: 0,
    }];
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;
    publish_repo_bundle(&service, "beta/repo", &documents, Some("rev-1")).await;

    service
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&["alpha/repo", "beta/repo"]))
        .await;
    let left_snapshot = service.status_with_repo_runtime().await;

    service.clear_all_in_memory_repo_runtime_for_test();
    service
        .synchronize_repo_runtime_for_test(&ready_repo_status_rows(&["beta/repo", "alpha/repo"]))
        .await;
    let right_snapshot = service.status_with_repo_runtime().await;

    assert_eq!(
        corpus_status(
            &left_snapshot,
            SearchCorpusKind::RepoEntity,
            "left repo entity row",
        ),
        corpus_status(
            &right_snapshot,
            SearchCorpusKind::RepoEntity,
            "right repo entity row",
        )
    );
    assert_eq!(
        corpus_status(
            &left_snapshot,
            SearchCorpusKind::RepoContentChunk,
            "left repo content row",
        ),
        corpus_status(
            &right_snapshot,
            SearchCorpusKind::RepoContentChunk,
            "right repo content row",
        )
    );
}

#[test]
fn synchronize_repo_runtime_replaces_previous_snapshot_entries() {
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/project/.data/search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );

    service.synchronize_repo_runtime(&RepoIndexStatusResponse {
        total: 2,
        active: 0,
        queued: 1,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 1,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: vec![
            repo_status_entry("alpha/repo", RepoIndexPhase::Ready),
            repo_status_entry("beta/repo", RepoIndexPhase::Queued),
        ],
    });
    assert_eq!(
        repo_phase(&service, "alpha/repo"),
        Some(RepoIndexPhase::Ready)
    );
    assert_eq!(
        repo_phase(&service, "beta/repo"),
        Some(RepoIndexPhase::Queued)
    );

    service.synchronize_repo_runtime(&RepoIndexStatusResponse {
        total: 1,
        active: 0,
        queued: 1,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 0,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: vec![repo_status_entry("beta/repo", RepoIndexPhase::Queued)],
    });

    assert_eq!(repo_phase(&service, "alpha/repo"), None);
    assert_eq!(
        repo_phase(&service, "beta/repo"),
        Some(RepoIndexPhase::Queued)
    );
}

#[tokio::test]
async fn local_corpus_ready_status_bootstraps_from_persisted_manifest_after_restart() {
    let temp_dir = temp_dir();
    let project_root = temp_dir.path().join("workspace");
    let storage_root = temp_dir.path().join("search_plane");
    std::fs::create_dir_all(project_root.join("docs"))
        .unwrap_or_else(|error| panic!("create docs dir: {error}"));
    std::fs::write(
        project_root.join("docs/intro.md"),
        "# Warm Start\n\nRestart should reuse the local publication.\n",
    )
    .unwrap_or_else(|error| panic!("write note: {error}"));
    let projects = vec![crate::gateway::studio::types::UiProjectConfig {
        name: "docs".to_string(),
        root: ".".to_string(),
        dirs: vec!["docs".to_string()],
    }];

    let writer = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root.clone(),
        unique_test_manifest_keyspace("local-bootstrap-writer"),
        SearchMaintenancePolicy::default(),
    );
    ok_or_panic(
        writer
            .publish_knowledge_sections_from_projects(
                project_root.as_path(),
                project_root.as_path(),
                &projects,
                "warm-start-fingerprint",
            )
            .await,
        "publish knowledge sections",
    );
    let written_status = writer
        .coordinator()
        .status_for(SearchCorpusKind::KnowledgeSection);
    let written_epoch = some_or_panic(
        written_status.active_epoch,
        "writer should publish a knowledge-section epoch",
    );

    let reader = SearchPlaneService::with_paths(
        project_root.clone(),
        storage_root,
        unique_test_manifest_keyspace("local-bootstrap-reader"),
        SearchMaintenancePolicy::default(),
    );
    let restored_status = reader
        .coordinator()
        .status_for(SearchCorpusKind::KnowledgeSection);

    assert_eq!(restored_status.phase, SearchPlanePhase::Ready);
    assert_eq!(restored_status.active_epoch, Some(written_epoch));
    assert_eq!(restored_status.row_count, written_status.row_count);
    assert_eq!(
        restored_status.fragment_count,
        written_status.fragment_count
    );
    assert_eq!(restored_status.fingerprint, written_status.fingerprint);
    assert_eq!(
        restored_status.build_finished_at,
        written_status.build_finished_at
    );
}

#[tokio::test]
async fn stale_repo_runtime_refresh_does_not_override_newer_generation() {
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/project/.data/search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );

    let stale_generation = service.advance_repo_runtime_generation_for_test();
    let current_generation = service.advance_repo_runtime_generation_for_test();
    assert!(current_generation > stale_generation);

    service
        .refresh_repo_runtime_cache_for_test(
            stale_generation,
            vec![SearchRepoRuntimeRecord {
                repo_id: "stale/repo".to_string(),
                phase: RepoIndexPhase::Ready,
                last_revision: Some("rev-1".to_string()),
                last_error: None,
                updated_at: Some("2026-03-22T12:00:00Z".to_string()),
            }],
        )
        .await;

    assert_eq!(repo_phase(&service, "stale/repo"), None);

    service
        .refresh_repo_runtime_cache_for_test(
            current_generation,
            vec![SearchRepoRuntimeRecord {
                repo_id: "fresh/repo".to_string(),
                phase: RepoIndexPhase::Ready,
                last_revision: Some("rev-2".to_string()),
                last_error: None,
                updated_at: Some("2026-03-22T12:05:00Z".to_string()),
            }],
        )
        .await;

    assert_eq!(
        repo_phase(&service, "fresh/repo"),
        Some(RepoIndexPhase::Ready)
    );
}
