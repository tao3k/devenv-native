use crate::repo_index::{RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::service::tests::status::repo_content::helpers::{repo_document, test_service};
use crate::search::service::tests::support::{
    assert_status_reason, corpus_status, publish_repo_bundle, repo_status_entry,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchPlanePhase,
};

#[tokio::test]
async fn status_with_repo_content_surfaces_ready_repo_tables() {
    let service = test_service();
    let documents = vec![repo_document(
        "src/lib.rs",
        "fn alpha() {}\nlet beta = alpha();\n",
        34,
        0,
    )];
    publish_repo_bundle(&service, "alpha/repo", &documents, Some("rev-1")).await;

    let status = service
        .status_with_repo_content(&RepoIndexStatusResponse {
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
        })
        .await;

    let repo_content = corpus_status(
        &status,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_eq!(repo_content.phase, SearchPlanePhase::Ready);
    assert!(repo_content.active_epoch.is_some());
    assert!(repo_content.staging_epoch.is_none());
    assert!(repo_content.row_count.unwrap_or_default() > 0);
    assert!(repo_content.fragment_count.unwrap_or_default() > 0);
    assert!(repo_content.fingerprint.is_some());
    assert!(repo_content.build_finished_at.is_some());
    assert!(repo_content.updated_at.is_some());
    assert!(repo_content.last_error.is_none());
    assert!(repo_content.issues.is_empty());
    assert!(repo_content.issue_summary.is_none());
    assert!(repo_content.status_reason.is_none());

    let repo_entity = corpus_status(
        &status,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_eq!(repo_entity.phase, SearchPlanePhase::Ready);
    assert!(repo_entity.active_epoch.is_some());
    assert!(repo_entity.staging_epoch.is_none());
    assert!(repo_entity.row_count.unwrap_or_default() > 0);
    assert!(repo_entity.fragment_count.unwrap_or_default() > 0);
    assert!(repo_entity.fingerprint.is_some());
    assert!(repo_entity.build_finished_at.is_some());
    assert!(repo_entity.updated_at.is_some());
    assert!(repo_entity.last_error.is_none());
    assert!(repo_entity.issues.is_empty());
    assert!(repo_entity.issue_summary.is_none());
    assert!(repo_entity.status_reason.is_none());
}

#[tokio::test]
async fn status_with_repo_content_reports_indexing_before_publish() {
    let service = test_service();

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
    assert!(repo_content.active_epoch.is_none());
    assert!(repo_content.staging_epoch.is_some());
    assert!(repo_content.row_count.is_none());
    assert!(repo_content.fragment_count.is_none());
    assert!(repo_content.fingerprint.is_none());
    assert!(repo_content.build_finished_at.is_none());
    assert!(repo_content.updated_at.is_some());
    assert!(repo_content.last_error.is_none());
    assert!(repo_content.issues.is_empty());
    assert!(repo_content.issue_summary.is_none());
    assert_status_reason(
        repo_content,
        SearchCorpusStatusReasonCode::WarmingUp,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        false,
    );

    let repo_entity = corpus_status(
        &status,
        SearchCorpusKind::RepoEntity,
        "repo entity row should exist",
    );
    assert_eq!(repo_entity.phase, SearchPlanePhase::Indexing);
    assert!(repo_entity.active_epoch.is_none());
    assert!(repo_entity.staging_epoch.is_some());
    assert!(repo_entity.row_count.is_none());
    assert!(repo_entity.fragment_count.is_none());
    assert!(repo_entity.fingerprint.is_none());
    assert!(repo_entity.build_finished_at.is_none());
    assert!(repo_entity.updated_at.is_some());
    assert!(repo_entity.last_error.is_none());
    assert!(repo_entity.issues.is_empty());
    assert!(repo_entity.issue_summary.is_none());
    assert_status_reason(
        repo_entity,
        SearchCorpusStatusReasonCode::WarmingUp,
        SearchCorpusStatusSeverity::Info,
        SearchCorpusStatusAction::Wait,
        false,
    );
}
