use std::sync::Arc;

use crate::repo_index::{RepoCodeDocument, RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::service::tests::support::{
    assert_status_reason, issue_summary, repo_status_entry,
};
use crate::search::{
    SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusStatus, SearchCorpusStatusAction,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchPlanePhase,
};

pub(super) fn sample_repo_documents() -> Vec<RepoCodeDocument> {
    vec![RepoCodeDocument {
        path: "src/lib.rs".to_string().into(),
        language: Some("rust".to_string()),
        contents: Arc::<str>::from("fn alpha() {}\nlet beta = alpha();\n"),
        size_bytes: 34,
        modified_unix_ms: 0,
    }]
}

pub(super) fn ready_repo_status(repo_id: &str) -> RepoIndexStatusResponse {
    RepoIndexStatusResponse {
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
        repos: vec![repo_status_entry(repo_id, RepoIndexPhase::Ready)],
    }
}

pub(super) fn assert_revision_mismatch_status(
    status: &SearchCorpusStatus,
    repo_id: &str,
    current_revision: &str,
    published_revision: &str,
) {
    assert_eq!(status.phase, SearchPlanePhase::Degraded);
    assert!(status.last_error.as_deref().is_some_and(|message| {
        message.contains(&format!("targets revision `{published_revision}`"))
    }));
    assert_eq!(status.issues.len(), 1);
    assert_eq!(
        status.issues[0].code,
        SearchCorpusIssueCode::PublishedRevisionMismatch
    );
    assert_eq!(status.issues[0].repo_id.as_deref(), Some(repo_id));
    assert_eq!(
        status.issues[0].current_revision.as_deref(),
        Some(current_revision)
    );
    assert_eq!(
        status.issues[0].published_revision.as_deref(),
        Some(published_revision)
    );
    assert!(status.issues[0].readable);

    let summary = issue_summary(status, "issue summary should be present");
    assert_eq!(summary.family, SearchCorpusIssueFamily::Revision);
    assert_eq!(
        summary.primary_code,
        SearchCorpusIssueCode::PublishedRevisionMismatch
    );
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.readable_issue_count, 1);
    assert_status_reason(
        status,
        SearchCorpusStatusReasonCode::PublishedRevisionMismatch,
        SearchCorpusStatusSeverity::Warning,
        SearchCorpusStatusAction::ResyncRepo,
        true,
    );
}

pub(super) fn assert_manifest_missing_status(
    status: &SearchCorpusStatus,
    repo_id: &str,
    current_revision: &str,
) {
    assert_eq!(status.phase, SearchPlanePhase::Failed);
    assert!(status.row_count.is_none());
    assert!(status.fragment_count.is_none());
    assert!(status.fingerprint.is_none());
    assert!(
        status
            .last_error
            .as_deref()
            .is_some_and(|message| message.contains("published state"))
    );
    assert_eq!(status.issues.len(), 1);
    assert_eq!(
        status.issues[0].code,
        SearchCorpusIssueCode::PublishedManifestMissing
    );
    assert_eq!(status.issues[0].repo_id.as_deref(), Some(repo_id));
    assert_eq!(
        status.issues[0].current_revision.as_deref(),
        Some(current_revision)
    );
    assert_eq!(status.issues[0].published_revision, None);
    assert!(!status.issues[0].readable);

    let summary = issue_summary(status, "issue summary should be present");
    assert_eq!(summary.family, SearchCorpusIssueFamily::Manifest);
    assert_eq!(
        summary.primary_code,
        SearchCorpusIssueCode::PublishedManifestMissing
    );
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.readable_issue_count, 0);
    assert_status_reason(
        status,
        SearchCorpusStatusReasonCode::PublishedManifestMissing,
        SearchCorpusStatusSeverity::Error,
        SearchCorpusStatusAction::ResyncRepo,
        false,
    );
}
