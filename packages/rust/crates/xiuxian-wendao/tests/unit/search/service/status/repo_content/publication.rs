use crate::repo_index::{RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::service::tests::status::helpers::{
    assert_manifest_missing_status, assert_revision_mismatch_status, ready_repo_status,
    sample_repo_documents,
};
use crate::search::service::tests::status::repo_content::helpers::test_service;
use crate::search::service::tests::support::{
    assert_status_reason, corpus_status, publish_repo_bundle, repo_status_entry,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchPlanePhase,
};

#[tokio::test]
async fn status_with_repo_content_keeps_published_metadata_while_repo_refreshes() {
    let service = test_service();
    let documents = sample_repo_documents();
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-0")).await;

    let status = service
        .status_with_repo_content(&RepoIndexStatusResponse {
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
            repos: vec![repo_status_entry("alpha/repo", RepoIndexPhase::Indexing)],
        })
        .await;

    let repo_content = corpus_status(
        &status,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_eq!(repo_content.phase, SearchPlanePhase::Indexing);
    assert!(repo_content.active_epoch.is_some());
    assert!(repo_content.staging_epoch.is_some());
    assert!(repo_content.row_count.unwrap_or_default() > 0);
    assert!(repo_content.fragment_count.unwrap_or_default() > 0);
    assert!(repo_content.fingerprint.is_some());
    assert!(repo_content.build_finished_at.is_some());
    assert!(repo_content.issues.is_empty());
    assert!(repo_content.issue_summary.is_none());
    assert_status_reason(
        repo_content,
        SearchCorpusStatusReasonCode::Refreshing,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        true,
    );

    let repo_entity = corpus_status(
        &status,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_eq!(repo_entity.phase, SearchPlanePhase::Indexing);
    assert!(repo_entity.active_epoch.is_some());
    assert!(repo_entity.staging_epoch.is_some());
    assert!(repo_entity.row_count.unwrap_or_default() > 0);
    assert!(repo_entity.fragment_count.unwrap_or_default() > 0);
    assert!(repo_entity.fingerprint.is_some());
    assert!(repo_entity.build_finished_at.is_some());
    assert!(repo_entity.last_error.is_none());
    assert!(repo_entity.issues.is_empty());
    assert!(repo_entity.issue_summary.is_none());
    assert_status_reason(
        repo_entity,
        SearchCorpusStatusReasonCode::Refreshing,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        true,
    );
}

#[tokio::test]
async fn status_with_repo_content_reports_revision_mismatch_for_ready_repo() {
    let service = test_service();
    let documents = sample_repo_documents();
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-0")).await;

    let status = service
        .status_with_repo_content(&ready_repo_status("alpha/repo"))
        .await;

    let repo_content = corpus_status(
        &status,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_revision_mismatch_status(repo_content, "alpha/repo", "rev-1", "rev-0");

    let repo_entity = corpus_status(
        &status,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_revision_mismatch_status(repo_entity, "alpha/repo", "rev-1", "rev-0");
}

#[tokio::test]
async fn status_with_repo_content_requires_published_state_even_when_disk_tables_exist() {
    let service = test_service();
    let documents = sample_repo_documents();
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;
    service.clear_repo_publications("alpha/repo");

    assert!(!service.has_published_repo_corpus(SearchCorpusKind::RepoEntity, "alpha/repo"));
    assert!(!service.has_published_repo_corpus(SearchCorpusKind::RepoContentChunk, "alpha/repo"));

    let status = service
        .status_with_repo_content(&ready_repo_status("alpha/repo"))
        .await;

    let repo_content = corpus_status(
        &status,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_manifest_missing_status(repo_content, "alpha/repo", "rev-1");

    let repo_entity = corpus_status(
        &status,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_manifest_missing_status(repo_entity, "alpha/repo", "rev-1");
}
