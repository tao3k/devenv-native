use std::path::PathBuf;

use crate::repo_index::RepoIndexPhase;
use crate::search::coordinator::SearchCompactionReason;
use crate::search::service::core::types::{
    QueuedRepoMaintenanceTask, RepoCompactionTask, RepoMaintenanceTask, RepoMaintenanceTaskKind,
    RepoPrewarmTask, SearchPlaneService,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatusAction, SearchCorpusStatusReasonCode,
    SearchCorpusStatusSeverity, SearchMaintenancePolicy, SearchMaintenanceStatus,
    SearchManifestKeyspace, SearchRepoCorpusRecord, SearchRepoPublicationInput,
    SearchRepoPublicationRecord, SearchRepoRuntimeRecord,
};

fn compaction_test_service(keyspace: &str) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        PathBuf::from("/tmp/search-plane"),
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    )
}

fn repo_entity_publication() -> SearchRepoPublicationRecord {
    SearchRepoPublicationRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        SearchRepoPublicationInput {
            table_name: "repo_entity_repo_alpha".to_string(),
            schema_version: SearchCorpusKind::RepoEntity.schema_version(),
            source_revision: Some("rev-1".to_string()),
            table_version_id: 7,
            row_count: 12,
            fragment_count: 4,
            published_at: "2026-03-24T12:34:56Z".to_string(),
        },
    )
}

fn ready_repo_record(maintenance: SearchMaintenanceStatus) -> SearchRepoCorpusRecord {
    SearchRepoCorpusRecord::new(
        SearchCorpusKind::RepoEntity,
        "alpha/repo",
        Some(SearchRepoRuntimeRecord {
            repo_id: "alpha/repo".to_string(),
            phase: RepoIndexPhase::Ready,
            last_revision: Some("rev-1".to_string()),
            last_error: None,
            updated_at: Some("2026-03-24T12:34:56Z".to_string()),
        }),
        Some(repo_entity_publication()),
    )
    .with_maintenance(Some(maintenance))
}

fn queued_prewarm(
    corpus: SearchCorpusKind,
    repo_id: &str,
    table_name: &str,
    projected_columns: &[&str],
    enqueue_sequence: u64,
) -> QueuedRepoMaintenanceTask {
    QueuedRepoMaintenanceTask {
        task: RepoMaintenanceTask::Prewarm(RepoPrewarmTask {
            corpus,
            repo_id: repo_id.to_string(),
            table_name: table_name.to_string(),
            projected_columns: projected_columns
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        }),
        enqueue_sequence,
    }
}

fn queued_compaction(
    corpus: SearchCorpusKind,
    repo_id: &str,
    publication_id: &str,
    table_name: &str,
    enqueue_sequence: u64,
    reason: SearchCompactionReason,
) -> QueuedRepoMaintenanceTask {
    QueuedRepoMaintenanceTask {
        task: RepoMaintenanceTask::Compaction(RepoCompactionTask {
            corpus,
            repo_id: repo_id.to_string(),
            publication_id: publication_id.to_string(),
            table_name: table_name.to_string(),
            row_count: 12,
            reason,
        }),
        enqueue_sequence,
    }
}

#[test]
fn annotate_runtime_status_preserves_repo_compaction_running_from_record_maintenance() {
    let service = compaction_test_service("xiuxian:test:search-plane:repo-compaction");
    let record = ready_repo_record(SearchMaintenanceStatus {
        compaction_running: true,
        compaction_pending: true,
        publish_count_since_compaction: 1,
        ..SearchMaintenanceStatus::default()
    });

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert!(status.maintenance.compaction_running);
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::Compacting);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(reason.readable);
}

#[test]
fn annotate_runtime_status_marks_repo_compaction_running_from_active_task() {
    let service = compaction_test_service("xiuxian:test:search-plane:repo-compaction-active");
    let record = ready_repo_record(SearchMaintenanceStatus {
        compaction_running: false,
        compaction_pending: true,
        publish_count_since_compaction: 1,
        ..SearchMaintenanceStatus::default()
    });
    service
        .repo_maintenance
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .active_task = Some((
        SearchCorpusKind::RepoEntity,
        "alpha/repo".to_string(),
        "publication-alpha".to_string(),
        RepoMaintenanceTaskKind::Compaction,
    ));

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert!(status.maintenance.compaction_running);
    assert_eq!(status.maintenance.compaction_queue_depth, 0);
    assert_eq!(status.maintenance.compaction_queue_position, None);
    assert!(!status.maintenance.compaction_queue_aged);
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::Compacting);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(reason.readable);
}

#[test]
fn annotate_runtime_status_surfaces_repo_compaction_queue_backlog() {
    let service = compaction_test_service("xiuxian:test:search-plane:repo-compaction-queue");
    let record = ready_repo_record(SearchMaintenanceStatus {
        compaction_pending: true,
        publish_count_since_compaction: 1,
        ..SearchMaintenanceStatus::default()
    });
    {
        let mut runtime = service
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.queue.push_back(queued_prewarm(
            SearchCorpusKind::RepoEntity,
            "beta/repo",
            "repo_entity_repo_beta",
            &["name"],
            0,
        ));
        runtime.queue.push_back(queued_compaction(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            "publication-alpha",
            "repo_entity_repo_alpha",
            1,
            SearchCompactionReason::PublishThreshold,
        ));
        runtime.queue.push_back(queued_compaction(
            SearchCorpusKind::RepoContentChunk,
            "gamma/repo",
            "publication-gamma",
            "repo_content_chunk_repo_gamma",
            0,
            SearchCompactionReason::PublishThreshold,
        ));
    }

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert!(!status.maintenance.compaction_running);
    assert_eq!(status.maintenance.compaction_queue_depth, 1);
    assert_eq!(status.maintenance.compaction_queue_position, Some(2));
    assert!(!status.maintenance.compaction_queue_aged);
    let reason = status
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should exist"));
    assert_eq!(reason.code, SearchCorpusStatusReasonCode::CompactionPending);
    assert_eq!(reason.severity, SearchCorpusStatusSeverity::Info);
    assert_eq!(reason.action, SearchCorpusStatusAction::Wait);
    assert!(reason.readable);
}

#[test]
fn annotate_runtime_status_surfaces_repo_compaction_queue_aging() {
    let service = compaction_test_service("xiuxian:test:search-plane:repo-compaction-aged");
    let record = ready_repo_record(SearchMaintenanceStatus {
        compaction_pending: true,
        publish_count_since_compaction: 1,
        ..SearchMaintenanceStatus::default()
    });
    {
        let mut runtime = service
            .repo_maintenance
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        runtime.next_enqueue_sequence = 4;
        runtime.queue.push_back(queued_compaction(
            SearchCorpusKind::RepoEntity,
            "alpha/repo",
            "publication-alpha",
            "repo_entity_repo_alpha",
            0,
            SearchCompactionReason::RowDeltaRatio,
        ));
    }

    let mut status =
        SearchPlaneService::synthesize_repo_table_status(&[record], SearchCorpusKind::RepoEntity);
    service.annotate_runtime_status(&mut status);

    assert_eq!(status.maintenance.compaction_queue_depth, 1);
    assert_eq!(status.maintenance.compaction_queue_position, Some(1));
    assert!(status.maintenance.compaction_queue_aged);
}
