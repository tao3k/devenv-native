use std::path::PathBuf;

use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::cache::SearchPlaneCache;
use crate::search::service::tests::support::{
    repo_status_entry, service_test_manifest_keyspace, some_or_panic, temp_dir,
};
use crate::search::{
    RepoSearchQueryCacheKeyInput, SearchCorpusKind, SearchMaintenancePolicy, SearchPlaneService,
    SearchPublicationStorageFormat, SearchRepoPublicationInput,
};

#[tokio::test]
async fn repo_search_query_cache_key_uses_synchronized_runtime_state() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );

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

    let ready_key = some_or_panic(
        service
            .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
                scope: "code_search",
                corpora: &[],
                repo_corpora: &[SearchCorpusKind::RepoEntity],
                repo_ids: &[String::from("alpha/repo")],
                query: "alpha",
                limit: 10,
                intent: Some("code_search"),
                repo_hint: Some("alpha/repo"),
            })
            .await,
        "cache key should exist",
    );

    service.synchronize_repo_runtime(&RepoIndexStatusResponse {
        total: 1,
        active: 1,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 1,
        ready: 0,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: Some("alpha/repo".to_string()),
        active_repo_ids: vec!["alpha/repo".to_string()],
        repos: vec![RepoIndexEntryStatus {
            last_revision: Some("rev-2".to_string()),
            ..repo_status_entry("alpha/repo", RepoIndexPhase::Indexing)
        }],
    });

    let refreshing_key = some_or_panic(
        service
            .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
                scope: "code_search",
                corpora: &[],
                repo_corpora: &[SearchCorpusKind::RepoEntity],
                repo_ids: &[String::from("alpha/repo")],
                query: "alpha",
                limit: 10,
                intent: Some("code_search"),
                repo_hint: Some("alpha/repo"),
            })
            .await,
        "cache key should exist",
    );

    assert_ne!(ready_key, refreshing_key);
}

#[tokio::test]
async fn repo_search_query_cache_key_normalizes_repo_id_order_and_duplicates() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );

    service.synchronize_repo_runtime(&RepoIndexStatusResponse {
        total: 2,
        active: 0,
        queued: 0,
        checking: 0,
        syncing: 0,
        indexing: 0,
        ready: 2,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 1,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: vec![
            repo_status_entry("alpha/repo", RepoIndexPhase::Ready),
            repo_status_entry("beta/repo", RepoIndexPhase::Ready),
        ],
    });

    let unordered = some_or_panic(
        service
            .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
                scope: "code_search",
                corpora: &[],
                repo_corpora: &[
                    SearchCorpusKind::RepoEntity,
                    SearchCorpusKind::RepoContentChunk,
                ],
                repo_ids: &[
                    String::from("beta/repo"),
                    String::from("alpha/repo"),
                    String::from("alpha/repo"),
                ],
                query: "solve",
                limit: 10,
                intent: Some("code_search"),
                repo_hint: None,
            })
            .await,
        "unordered repo-set cache key should exist",
    );
    let normalized = some_or_panic(
        service
            .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
                scope: "code_search",
                corpora: &[],
                repo_corpora: &[
                    SearchCorpusKind::RepoEntity,
                    SearchCorpusKind::RepoContentChunk,
                ],
                repo_ids: &[String::from("alpha/repo"), String::from("beta/repo")],
                query: "solve",
                limit: 10,
                intent: Some("code_search"),
                repo_hint: None,
            })
            .await,
        "normalized repo-set cache key should exist",
    );

    assert_eq!(unordered, normalized);
}

#[tokio::test]
async fn recorded_repo_publication_remains_available_by_revision() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );

    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            SearchRepoPublicationInput {
                table_name: "repo_entity_alpha_repo".to_string(),
                schema_version: 1,
                source_revision: Some("rev-clean-build".to_string()),
                table_version_id: 7,
                row_count: 5,
                fragment_count: 1,
                published_at: "2026-04-06T00:00:00Z".to_string(),
            },
            SearchPublicationStorageFormat::Lance,
        )
        .await;

    let publication = some_or_panic(
        service
            .repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                "rev-clean-build",
            )
            .await,
        "publication should be retrievable by revision",
    );

    assert_eq!(publication.repo_id, "alpha/repo");
    assert_eq!(
        publication.source_revision.as_deref(),
        Some("rev-clean-build")
    );
    assert_eq!(
        publication.storage_format,
        SearchPublicationStorageFormat::Lance
    );
}

#[tokio::test]
async fn readable_repo_publication_prefers_latest_persisted_record_when_revision_cache_is_empty() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let cache = SearchPlaneCache::for_tests(keyspace.clone());
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace,
        SearchMaintenancePolicy::default(),
        cache.clone(),
    );

    service
        .record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            SearchRepoPublicationInput {
                table_name: "repo_entity_alpha_repo_rev_1".to_string(),
                schema_version: SearchCorpusKind::RepoEntity.schema_version(),
                source_revision: Some("rev-1".to_string()),
                table_version_id: 1,
                row_count: 5,
                fragment_count: 1,
                published_at: "2026-04-06T00:00:01Z".to_string(),
            },
            SearchPublicationStorageFormat::Parquet,
        )
        .await;

    cache
        .delete_repo_publication_revision_cache(SearchCorpusKind::RepoEntity, "alpha/repo")
        .await;
    service.clear_all_in_memory_repo_corpus_records_for_test();

    let publication = some_or_panic(
        service
            .readable_repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                "rev-1",
            )
            .await,
        "readable publication should resolve from latest record",
    );

    assert_eq!(publication.repo_id, "alpha/repo");
    assert_eq!(publication.source_revision.as_deref(), Some("rev-1"));
    assert_eq!(
        publication.storage_format,
        SearchPublicationStorageFormat::Parquet
    );
}

#[tokio::test]
async fn refresh_repo_backed_publications_for_revision_advances_both_repo_corpora() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let service = SearchPlaneService::with_runtime(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace.clone(),
        SearchMaintenancePolicy::default(),
        SearchPlaneCache::for_tests(keyspace),
    );

    for corpus in [
        SearchCorpusKind::RepoEntity,
        SearchCorpusKind::RepoContentChunk,
    ] {
        service
            .record_repo_publication_input_with_storage_format(
                corpus,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: format!("{corpus}_alpha_repo_rev_1"),
                    schema_version: corpus.schema_version(),
                    source_revision: Some("rev-1".to_string()),
                    table_version_id: 1,
                    row_count: 1,
                    fragment_count: 1,
                    published_at: "2026-04-06T00:00:01Z".to_string(),
                },
                SearchPublicationStorageFormat::Parquet,
            )
            .await;
    }

    assert!(
        service
            .refresh_repo_backed_publications_for_revision("alpha/repo", "rev-2")
            .await
    );

    for corpus in [
        SearchCorpusKind::RepoEntity,
        SearchCorpusKind::RepoContentChunk,
    ] {
        let publication = some_or_panic(
            service
                .readable_repo_publication_for_revision(corpus, "alpha/repo", "rev-2")
                .await,
            "refreshed publication should resolve by new revision",
        );
        assert_eq!(publication.repo_id, "alpha/repo");
        assert_eq!(publication.source_revision.as_deref(), Some("rev-2"));
        assert_eq!(
            publication.storage_format,
            SearchPublicationStorageFormat::Parquet
        );
    }
}

#[tokio::test]
async fn recorded_repo_publication_revision_retention_trims_old_entries() {
    let temp_dir = temp_dir();
    let keyspace = service_test_manifest_keyspace();
    let service = SearchPlaneService::with_test_cache_and_revision_retention(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        keyspace,
        SearchMaintenancePolicy::default(),
        2,
    );

    for (table_version_id, revision) in [(1, "rev-1"), (2, "rev-2"), (3, "rev-3")] {
        service
            .record_repo_publication_input_with_storage_format(
                SearchCorpusKind::RepoEntity,
                "alpha/repo",
                SearchRepoPublicationInput {
                    table_name: format!("repo_entity_alpha_repo_{revision}"),
                    schema_version: 1,
                    source_revision: Some(revision.to_string()),
                    table_version_id,
                    row_count: 5,
                    fragment_count: 1,
                    published_at: format!("2026-04-06T00:00:0{table_version_id}Z"),
                },
                SearchPublicationStorageFormat::Lance,
            )
            .await;
    }

    assert!(
        service
            .repo_publication_for_revision(SearchCorpusKind::RepoEntity, "alpha/repo", "rev-1")
            .await
            .is_none()
    );
    assert!(
        service
            .repo_publication_for_revision(SearchCorpusKind::RepoEntity, "alpha/repo", "rev-2")
            .await
            .is_some()
    );
    assert!(
        service
            .repo_publication_for_revision(SearchCorpusKind::RepoEntity, "alpha/repo", "rev-3")
            .await
            .is_some()
    );
}
