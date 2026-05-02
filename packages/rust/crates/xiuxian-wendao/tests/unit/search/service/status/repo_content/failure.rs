use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::service::tests::status::repo_content::helpers::{repo_document, test_service};
use crate::search::service::tests::support::{
    assert_status_reason, corpus_status, issue_summary, publish_repo_bundle, repo_status_entry,
};
use crate::search::{
    SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusKind, SearchCorpusStatusAction,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchPlanePhase,
};

#[tokio::test]
async fn status_with_repo_content_reports_repo_failure_issue_while_rows_remain_readable() {
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
            ready: 0,
            unsupported: 0,
            failed: 1,
            target_concurrency: 1,
            max_concurrency: 1,
            sync_concurrency_limit: 1,
            current_repo_id: None,
            active_repo_ids: Vec::new(),
            repos: vec![RepoIndexEntryStatus {
                last_error: Some("git fetch failed".to_string()),
                ..repo_status_entry("alpha/repo", RepoIndexPhase::Failed)
            }],
        })
        .await;

    let repo_content = corpus_status(
        &status,
        SearchCorpusKind::RepoContentChunk,
        "repo content row should exist",
    );
    assert_eq!(repo_content.phase, SearchPlanePhase::Degraded);
    assert_eq!(repo_content.issues.len(), 1);
    assert_eq!(
        repo_content.issues[0].code,
        SearchCorpusIssueCode::RepoIndexFailed
    );
    assert!(repo_content.issues[0].readable);
    assert_eq!(
        repo_content.issues[0].published_revision.as_deref(),
        Some("rev-1")
    );
    let repo_content_summary = issue_summary(repo_content, "issue summary should be present");
    assert_eq!(
        repo_content_summary.family,
        SearchCorpusIssueFamily::RepoSync
    );
    assert_eq!(
        repo_content_summary.primary_code,
        SearchCorpusIssueCode::RepoIndexFailed
    );
    assert_eq!(repo_content_summary.issue_count, 1);
    assert_eq!(repo_content_summary.readable_issue_count, 1);
    assert_status_reason(
        repo_content,
        SearchCorpusStatusReasonCode::RepoIndexFailed,
        SearchCorpusStatusSeverity::Warning,
        SearchCorpusStatusAction::InspectRepoSync,
        true,
    );
    assert!(
        repo_content
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("git fetch failed"))
    );
}
