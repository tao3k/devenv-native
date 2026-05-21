use std::path::PathBuf;

use crate::search::repo_content_chunk_file_fingerprints;
use crate::search::repo_staging::versioned_repo_table_name;
use crate::search::service::core::RepoMaintenanceTaskKind;
use crate::search::service::tests::support::{
    ok_or_panic, publish_repo_bundle, sample_repo_documents, service_test_manifest_keyspace,
    some_or_panic, temp_dir,
};
use crate::search::{SearchCorpusKind, SearchMaintenancePolicy, SearchPlaneService};

#[tokio::test]
async fn repo_publication_runs_prewarm_via_maintenance_task_and_releases_repo_slot() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );

    publish_repo_bundle(
        &service,
        "alpha/repo",
        &sample_repo_documents(),
        Some("rev-1"),
    )
    .await;

    let entity = some_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, "alpha/repo")
            .await,
        "repo entity record should exist",
    );
    let content = some_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "repo content record should exist",
    );
    assert!(
        entity
            .maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.last_prewarmed_at.as_ref())
            .is_some()
    );
    assert!(
        !entity
            .maintenance
            .as_ref()
            .is_some_and(|maintenance| maintenance.prewarm_running)
    );
    assert!(
        content
            .maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.last_prewarmed_at.as_ref())
            .is_some()
    );
    assert!(
        !content
            .maintenance
            .as_ref()
            .is_some_and(|maintenance| maintenance.prewarm_running)
    );
    assert!(
        service
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .in_flight
            .is_empty()
    );
}

#[tokio::test]
async fn repo_publication_does_not_skip_prewarm_when_slot_is_already_claimed() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    );
    let repo_id = "alpha/repo";
    let revision = Some("rev-1");
    let documents = sample_repo_documents();
    let file_fingerprints = repo_content_chunk_file_fingerprints(&documents);
    let table_name = versioned_repo_table_name(
        SearchPlaneService::repo_content_chunk_table_name(repo_id).as_str(),
        repo_id,
        &file_fingerprints,
        revision,
        SearchCorpusKind::RepoContentChunk,
        1,
    );
    service
        .repo_maintenance
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .in_flight
        .insert((
            SearchCorpusKind::RepoContentChunk,
            repo_id.to_string(),
            table_name,
            RepoMaintenanceTaskKind::Prewarm,
        ));

    ok_or_panic(
        service
            .publish_repo_content_chunks_with_revision(repo_id, &documents, revision)
            .await,
        "publish repo content chunks",
    );

    let content = some_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
            .await,
        "repo content record should exist",
    );
    assert!(
        content
            .maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.last_prewarmed_at.as_ref())
            .is_some()
    );
}

#[tokio::test]
async fn repo_publication_keeps_compaction_disabled_for_parquet_publications() {
    let temp_dir = temp_dir();
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy {
            publish_count_threshold: 1,
            row_delta_ratio_threshold: 1.0,
        },
    );

    publish_repo_bundle(
        &service,
        "alpha/repo",
        &sample_repo_documents(),
        Some("rev-1"),
    )
    .await;
    let entity = some_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, "alpha/repo")
            .await,
        "repo entity record should exist",
    );
    let content = some_or_panic(
        service
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, "alpha/repo")
            .await,
        "repo content record should exist",
    );
    let entity_maintenance = some_or_panic(entity.maintenance.as_ref(), "entity maintenance");
    let content_maintenance = some_or_panic(content.maintenance.as_ref(), "content maintenance");
    assert!(!entity_maintenance.compaction_running);
    assert!(!entity_maintenance.compaction_pending);
    assert_eq!(entity_maintenance.publish_count_since_compaction, 0);
    assert!(entity_maintenance.last_compacted_at.is_none());
    assert!(entity_maintenance.last_compaction_reason.is_none());
    assert!(entity_maintenance.last_compacted_row_count.is_none());
    assert!(!content_maintenance.compaction_running);
    assert!(!content_maintenance.compaction_pending);
    assert_eq!(content_maintenance.publish_count_since_compaction, 0);
    assert!(content_maintenance.last_compacted_at.is_none());
    assert!(content_maintenance.last_compaction_reason.is_none());
    assert!(content_maintenance.last_compacted_row_count.is_none());
}
