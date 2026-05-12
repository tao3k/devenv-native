//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::{
    SearchCorpusIssue, SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusIssueSummary,
    SearchCorpusKind, SearchCorpusStatus, SearchCorpusStatusAction, SearchCorpusStatusReason,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchPlanePhase,
    SearchRepoPublicationRecord,
};

pub(crate) fn repo_content_phase(
    has_ready_tables: bool,
    has_active_work: bool,
    has_failures: bool,
) -> SearchPlanePhase {
    if has_active_work {
        return SearchPlanePhase::Indexing;
    }
    if has_ready_tables && has_failures {
        return SearchPlanePhase::Degraded;
    }
    if has_ready_tables {
        return SearchPlanePhase::Ready;
    }
    if has_failures {
        return SearchPlanePhase::Failed;
    }
    SearchPlanePhase::Idle
}

pub(crate) fn update_latest_timestamp(target: &mut Option<String>, candidate: Option<&str>) {
    let Some(candidate) = candidate else {
        return;
    };
    if target.as_deref().is_none_or(|current| current < candidate) {
        *target = Some(candidate.to_string());
    }
}

pub(crate) fn annotate_status_reason(status: &mut SearchCorpusStatus) {
    status.status_reason = derive_status_reason(status);
}

pub(crate) fn join_issue_messages(issues: &[SearchCorpusIssue]) -> Option<String> {
    if issues.is_empty() {
        return None;
    }
    Some(
        issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join("; "),
    )
}

pub(crate) fn derive_status_reason(
    status: &SearchCorpusStatus,
) -> Option<SearchCorpusStatusReason> {
    if let Some(summary) = status.issue_summary.as_ref() {
        let readable = status_is_readable(status);
        return Some(SearchCorpusStatusReason {
            code: reason_code_for_issue(summary.primary_code),
            severity: reason_severity_for_issue(summary.primary_code, readable),
            action: reason_action_for_issue(summary.primary_code),
            readable,
        });
    }

    match status.phase {
        SearchPlanePhase::Indexing => Some(SearchCorpusStatusReason {
            code: if status_is_readable(status) {
                SearchCorpusStatusReasonCode::Refreshing
            } else if status.maintenance.prewarm_running || status_has_prewarmed_staging(status) {
                SearchCorpusStatusReasonCode::Prewarming
            } else {
                SearchCorpusStatusReasonCode::WarmingUp
            },
            severity: SearchCorpusStatusSeverity::Info,
            action: SearchCorpusStatusAction::Wait,
            readable: status_is_readable(status),
        }),
        SearchPlanePhase::Failed => {
            let readable = status_is_readable(status);
            Some(SearchCorpusStatusReason {
                code: SearchCorpusStatusReasonCode::BuildFailed,
                severity: if readable {
                    SearchCorpusStatusSeverity::Warning
                } else {
                    SearchCorpusStatusSeverity::Error
                },
                action: SearchCorpusStatusAction::RetryBuild,
                readable,
            })
        }
        SearchPlanePhase::Ready => {
            if status.maintenance.compaction_running {
                Some(SearchCorpusStatusReason {
                    code: SearchCorpusStatusReasonCode::Compacting,
                    severity: SearchCorpusStatusSeverity::Info,
                    action: SearchCorpusStatusAction::Wait,
                    readable: true,
                })
            } else {
                status
                    .maintenance
                    .compaction_pending
                    .then_some(SearchCorpusStatusReason {
                        code: SearchCorpusStatusReasonCode::CompactionPending,
                        severity: SearchCorpusStatusSeverity::Info,
                        action: SearchCorpusStatusAction::Wait,
                        readable: true,
                    })
            }
        }
        SearchPlanePhase::Idle | SearchPlanePhase::Degraded => None,
    }
}

fn status_has_prewarmed_staging(status: &SearchCorpusStatus) -> bool {
    status
        .staging_epoch
        .zip(status.maintenance.last_prewarmed_epoch)
        .is_some_and(|(staging_epoch, prewarmed_epoch)| staging_epoch == prewarmed_epoch)
}

pub(crate) fn summarize_issues(issues: &[SearchCorpusIssue]) -> Option<SearchCorpusIssueSummary> {
    let first = issues.first()?;
    let mut family = issue_family(first.code);
    let mut primary_code = first.code;
    let mut readable_issue_count = usize::from(first.readable);
    for issue in issues.iter().skip(1) {
        let current_family = issue_family(issue.code);
        if family != current_family {
            family = SearchCorpusIssueFamily::Mixed;
        }
        if issue_priority(issue.code) < issue_priority(primary_code) {
            primary_code = issue.code;
        }
        if issue.readable {
            readable_issue_count = readable_issue_count.saturating_add(1);
        }
    }
    Some(SearchCorpusIssueSummary {
        family,
        primary_code,
        issue_count: issues.len(),
        readable_issue_count,
    })
}

pub(crate) fn status_is_readable(status: &SearchCorpusStatus) -> bool {
    status.active_epoch.is_some()
        || status.row_count.is_some()
        || matches!(
            status.phase,
            SearchPlanePhase::Ready | SearchPlanePhase::Degraded
        )
}

pub(crate) fn reason_code_for_issue(code: SearchCorpusIssueCode) -> SearchCorpusStatusReasonCode {
    match code {
        SearchCorpusIssueCode::PublishedManifestMissing => {
            SearchCorpusStatusReasonCode::PublishedManifestMissing
        }
        SearchCorpusIssueCode::PublishedRevisionMissing => {
            SearchCorpusStatusReasonCode::PublishedRevisionMissing
        }
        SearchCorpusIssueCode::PublishedRevisionMismatch => {
            SearchCorpusStatusReasonCode::PublishedRevisionMismatch
        }
        SearchCorpusIssueCode::RepoIndexFailed => SearchCorpusStatusReasonCode::RepoIndexFailed,
    }
}

pub(crate) fn reason_action_for_issue(code: SearchCorpusIssueCode) -> SearchCorpusStatusAction {
    match code {
        SearchCorpusIssueCode::PublishedManifestMissing
        | SearchCorpusIssueCode::PublishedRevisionMissing
        | SearchCorpusIssueCode::PublishedRevisionMismatch => SearchCorpusStatusAction::ResyncRepo,
        SearchCorpusIssueCode::RepoIndexFailed => SearchCorpusStatusAction::InspectRepoSync,
    }
}

pub(crate) fn reason_severity_for_issue(
    code: SearchCorpusIssueCode,
    readable: bool,
) -> SearchCorpusStatusSeverity {
    match code {
        SearchCorpusIssueCode::PublishedManifestMissing
        | SearchCorpusIssueCode::PublishedRevisionMissing
        | SearchCorpusIssueCode::PublishedRevisionMismatch
        | SearchCorpusIssueCode::RepoIndexFailed => {
            if readable {
                SearchCorpusStatusSeverity::Warning
            } else {
                SearchCorpusStatusSeverity::Error
            }
        }
    }
}

pub(crate) fn issue_family(code: SearchCorpusIssueCode) -> SearchCorpusIssueFamily {
    match code {
        SearchCorpusIssueCode::PublishedManifestMissing
        | SearchCorpusIssueCode::PublishedRevisionMissing => SearchCorpusIssueFamily::Manifest,
        SearchCorpusIssueCode::PublishedRevisionMismatch => SearchCorpusIssueFamily::Revision,
        SearchCorpusIssueCode::RepoIndexFailed => SearchCorpusIssueFamily::RepoSync,
    }
}

pub(crate) fn issue_priority(code: SearchCorpusIssueCode) -> u8 {
    match code {
        SearchCorpusIssueCode::PublishedManifestMissing => 0,
        SearchCorpusIssueCode::PublishedRevisionMissing => 1,
        SearchCorpusIssueCode::PublishedRevisionMismatch => 2,
        SearchCorpusIssueCode::RepoIndexFailed => 3,
    }
}

pub(crate) fn repo_manifest_missing_issue(
    corpus: SearchCorpusKind,
    repo: &RepoIndexEntryStatus,
) -> SearchCorpusIssue {
    SearchCorpusIssue {
        code: SearchCorpusIssueCode::PublishedManifestMissing,
        readable: false,
        repo_id: Some(repo.repo_id.clone()),
        current_revision: repo.last_revision.clone(),
        published_revision: None,
        message: format!(
            "{}: published state for {} is missing",
            repo.repo_id,
            corpus.as_str()
        ),
    }
}

pub(crate) fn repo_index_failure_issue(
    repo: &RepoIndexEntryStatus,
    publication: Option<&SearchRepoPublicationRecord>,
) -> Option<SearchCorpusIssue> {
    let message = repo.last_error.as_ref()?.clone();
    Some(SearchCorpusIssue {
        code: SearchCorpusIssueCode::RepoIndexFailed,
        readable: publication.is_some(),
        repo_id: Some(repo.repo_id.clone()),
        current_revision: repo.last_revision.clone(),
        published_revision: publication.and_then(|publication| publication.source_revision.clone()),
        message: format!("{}: {message}", repo.repo_id),
    })
}

pub(crate) fn repo_publication_consistency_issue(
    corpus: SearchCorpusKind,
    repo: &RepoIndexEntryStatus,
    publication: &SearchRepoPublicationRecord,
) -> Option<SearchCorpusIssue> {
    if repo.phase != RepoIndexPhase::Ready {
        return None;
    }
    let current_revision = repo
        .last_revision
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let published_revision = publication
        .source_revision
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    if current_revision.is_empty() && published_revision.is_empty() {
        return None;
    }
    if published_revision.is_empty() {
        return Some(SearchCorpusIssue {
            code: SearchCorpusIssueCode::PublishedRevisionMissing,
            readable: true,
            repo_id: Some(repo.repo_id.clone()),
            current_revision: repo.last_revision.clone(),
            published_revision: publication.source_revision.clone(),
            message: format!(
                "{}: published state for {} is missing source revision while repo is ready at `{}`",
                repo.repo_id,
                corpus.as_str(),
                current_revision
            ),
        });
    }
    if current_revision.is_empty() || current_revision == published_revision {
        return None;
    }
    Some(SearchCorpusIssue {
        code: SearchCorpusIssueCode::PublishedRevisionMismatch,
        readable: true,
        repo_id: Some(repo.repo_id.clone()),
        current_revision: repo.last_revision.clone(),
        published_revision: publication.source_revision.clone(),
        message: format!(
            "{}: published state for {} targets revision `{published_revision}` but repo is ready at `{current_revision}`",
            repo.repo_id,
            corpus.as_str()
        ),
    })
}
