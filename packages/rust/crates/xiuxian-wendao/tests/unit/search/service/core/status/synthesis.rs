use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::service::core::types::SearchPlaneService;
use crate::search::service::helpers::repo_corpus_staging_epoch;
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchMaintenanceStatus, SearchPlanePhase, SearchRepoCorpusRecord,
    SearchRepoPublicationInput, SearchRepoPublicationRecord, SearchRepoRuntimeRecord,
};

#[test]
fn synthesize_repo_status_marks_indexing_corpus_as_prewarming_when_staging_was_prewarmed() {
    let runtime_status = RepoIndexEntryStatus {
        repo_id: "alpha/repo".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("rev-2".to_string()),
        updated_at: Some("2026-03-24T12:34:56Z".to_string()),
        attempt_count: 1,
    };
    let staging_epoch = repo_corpus_staging_epoch(
        SearchCorpusKind::RepoEntity,
        std::slice::from_ref(&runtime_status),
        None,
    )
    .unwrap_or_else(|| panic!("staging epoch should exist"));
    let record = SearchRepoCorpusRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        Some(SearchRepoRuntimeRecord::from_status(&runtime_status)),
        None,
    )
    .with_maintenance(Some(SearchMaintenanceStatus {
        last_prewarmed_at: Some("2026-03-24T12:34:57Z".to_string()),
        last_prewarmed_epoch: Some(staging_epoch),
        ..SearchMaintenanceStatus::default()
    }));

    let status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);

    assert_eq!(status.phase, SearchPlanePhase::Indexing);
    assert_eq!(status.staging_epoch, Some(staging_epoch));
    assert_eq!(status.maintenance.last_prewarmed_epoch, Some(staging_epoch));
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::Prewarming);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(!reason.readable);
}

#[test]
fn synthesize_repo_status_marks_indexing_corpus_as_prewarming_when_prewarm_is_running() {
    let runtime_status = RepoIndexEntryStatus {
        repo_id: "alpha/repo".to_string(),
        phase: RepoIndexPhase::Indexing,
        queue_position: None,
        last_error: None,
        last_revision: Some("rev-2".to_string()),
        updated_at: Some("2026-03-24T12:34:56Z".to_string()),
        attempt_count: 1,
    };
    let record = SearchRepoCorpusRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        Some(SearchRepoRuntimeRecord::from_status(&runtime_status)),
        None,
    )
    .with_maintenance(Some(SearchMaintenanceStatus {
        prewarm_running: true,
        ..SearchMaintenanceStatus::default()
    }));

    let status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);

    assert_eq!(status.phase, SearchPlanePhase::Indexing);
    assert!(status.maintenance.prewarm_running);
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::Prewarming);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(!reason.readable);
}

#[test]
fn synthesize_repo_status_is_stable_for_reordered_ready_published_records() {
    let alpha =
        ready_published_repo_record("alpha/repo", "rev-alpha", 11, 3, 1, "2026-03-24T12:34:56Z");
    let beta =
        ready_published_repo_record("beta/repo", "rev-beta", 23, 5, 2, "2026-03-24T12:35:56Z");

    let left = SearchPlaneService::synthesize_repo_table_status(
        &[alpha.clone(), beta.clone()],
        SearchCorpusKind::RepoEntity,
    );
    let right = SearchPlaneService::synthesize_repo_table_status(
        &[beta, alpha],
        SearchCorpusKind::RepoEntity,
    );

    assert_eq!(left, right);
    assert_eq!(left.phase, SearchPlanePhase::Ready);
    assert_eq!(left.row_count, Some(8));
    assert_eq!(left.fragment_count, Some(3));
    assert!(
        left.active_epoch.is_some(),
        "expected synthesized active epoch to exist"
    );
    assert!(
        left.fingerprint.is_some(),
        "expected synthesized fingerprint to exist"
    );
}

fn ready_published_repo_record(
    repo_id: &str,
    revision: &str,
    table_version_id: u64,
    row_count: u64,
    fragment_count: u64,
    published_at: &str,
) -> SearchRepoCorpusRecord {
    let runtime_status = RepoIndexEntryStatus {
        repo_id: repo_id.to_string(),
        phase: RepoIndexPhase::Ready,
        queue_position: None,
        last_error: None,
        last_revision: Some(revision.to_string()),
        updated_at: Some(published_at.to_string()),
        attempt_count: 1,
    };
    let publication = SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        repo_id,
        SearchRepoPublicationInput {
            table_name: format!("repo_entity_{}", repo_id.replace('/', "_")),
            schema_version: SearchCorpusKind::RepoEntity.schema_version(),
            source_revision: Some(revision.to_string()),
            table_version_id,
            row_count,
            fragment_count,
            published_at: published_at.to_string(),
        },
    );
    SearchRepoCorpusRecord::new(
        SearchCorpusKind::RepoEntity,
        repo_id,
        Some(SearchRepoRuntimeRecord::from_status(&runtime_status)),
        Some(publication),
    )
}
