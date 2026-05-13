use std::path::PathBuf;

use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::service::core::types::{RepoMaintenanceTaskKind, SearchPlaneService};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchRepoCorpusRecord, SearchRepoRuntimeRecord,
};

#[test]
fn annotate_runtime_status_marks_repo_prewarm_running_from_active_task() {
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:search-plane:repo-prewarm-active"),
        SearchMaintenancePolicy::default(),
    );
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
    );
    service
        .repo_maintenance
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .active_task = Some((
        SearchCorpusKind::RepoEntity,
        "alpha/repo".to_string(),
        "repo_entity_repo_alpha".to_string(),
        RepoMaintenanceTaskKind::Prewarm,
    ));

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert!(status.maintenance.prewarm_running);
    assert_eq!(status.maintenance.prewarm_queue_depth, 0);
    assert_eq!(status.maintenance.prewarm_queue_position, None);
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
fn annotate_runtime_status_surfaces_repo_prewarm_queue_backlog() {
    let service = SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/search-plane"),
        SearchManifestKeyspace::new("xiuxian:test:search-plane:repo-prewarm-queue"),
        SearchMaintenancePolicy::default(),
    );
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
    );
    {
        let mut runtime = service
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.queue.push_back(
            crate::search::service::core::types::QueuedRepoMaintenanceTask {
                task: crate::search::service::core::types::RepoMaintenanceTask::Compaction(
                    crate::search::service::core::types::RepoCompactionTask {
                        corpus: SearchCorpusKind::RepoEntity,
                        repo_id: "beta/repo".to_string(),
                        publication_id: "publication-beta".to_string(),
                        table_name: "repo_entity_repo_beta".to_string(),
                        row_count: 12,
                        reason:
                            crate::search::coordinator::SearchCompactionReason::PublishThreshold,
                    },
                ),
                enqueue_sequence: 0,
            },
        );
        runtime.queue.push_back(
            crate::search::service::core::types::QueuedRepoMaintenanceTask {
                task: crate::search::service::core::types::RepoMaintenanceTask::Prewarm(
                    crate::search::service::core::types::RepoPrewarmTask {
                        corpus: SearchCorpusKind::RepoEntity,
                        repo_id: "alpha/repo".to_string(),
                        table_name: "repo_entity_repo_alpha".to_string(),
                        projected_columns: vec!["name".to_string()],
                    },
                ),
                enqueue_sequence: 1,
            },
        );
    }

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert!(!status.maintenance.prewarm_running);
    assert_eq!(status.maintenance.prewarm_queue_depth, 1);
    assert_eq!(status.maintenance.prewarm_queue_position, Some(2));
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::WarmingUp);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(!reason.readable);
}
