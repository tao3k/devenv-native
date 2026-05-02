use crate::gateway::studio::types::search_index::{
    SearchIndexStatusAction, SearchIndexStatusReasonCode, SearchIndexStatusResponse,
    SearchIndexStatusSeverity,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchCorpusStatusAction, SearchCorpusStatusReason,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchMaintenanceStatus,
    SearchPlanePhase, SearchPlaneStatusSnapshot,
};

use super::helpers::{corpus_status_reason, status_reason};

#[test]
fn response_status_reason_prefers_blocking_error_over_warning_and_info() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Failed;
    local_symbol.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::BuildFailed,
        severity: SearchCorpusStatusSeverity::Error,
        action: SearchCorpusStatusAction::RetryBuild,
        readable: false,
    });

    let mut knowledge = SearchCorpusStatus::new(SearchCorpusKind::KnowledgeSection);
    knowledge.phase = SearchPlanePhase::Ready;
    knowledge.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 2,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };
    knowledge.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::CompactionPending,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: true,
    });

    let mut repo_entity = SearchCorpusStatus::new(SearchCorpusKind::RepoEntity);
    repo_entity.phase = SearchPlanePhase::Degraded;
    repo_entity.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::PublishedRevisionMismatch,
        severity: SearchCorpusStatusSeverity::Warning,
        action: SearchCorpusStatusAction::ResyncRepo,
        readable: true,
    });

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, knowledge, repo_entity],
    });

    let aggregate_reason = status_reason(&response);
    assert_eq!(
        aggregate_reason.code,
        SearchIndexStatusReasonCode::BuildFailed
    );
    assert_eq!(aggregate_reason.severity, SearchIndexStatusSeverity::Error);
    assert_eq!(aggregate_reason.action, SearchIndexStatusAction::RetryBuild);
    assert_eq!(aggregate_reason.affected_corpus_count, 3);
    assert_eq!(aggregate_reason.readable_corpus_count, 2);
    assert_eq!(aggregate_reason.blocking_corpus_count, 1);
}

#[tokio::test]
async fn diagnostics_response_status_reason_prefers_blocking_error_over_warning_and_info() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Failed;
    local_symbol.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::BuildFailed,
        severity: SearchCorpusStatusSeverity::Error,
        action: SearchCorpusStatusAction::RetryBuild,
        readable: false,
    });

    let mut knowledge = SearchCorpusStatus::new(SearchCorpusKind::KnowledgeSection);
    knowledge.phase = SearchPlanePhase::Ready;
    knowledge.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 2,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };
    knowledge.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::CompactionPending,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: true,
    });

    let mut repo_entity = SearchCorpusStatus::new(SearchCorpusKind::RepoEntity);
    repo_entity.phase = SearchPlanePhase::Degraded;
    repo_entity.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::PublishedRevisionMismatch,
        severity: SearchCorpusStatusSeverity::Warning,
        action: SearchCorpusStatusAction::ResyncRepo,
        readable: true,
    });

    let snapshot = SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, knowledge, repo_entity],
    };

    let response = SearchIndexStatusResponse::from_snapshot_with_diagnostics(&snapshot).await;
    let baseline = SearchIndexStatusResponse::from(&snapshot);

    assert_eq!(response, baseline);

    let aggregate_reason = status_reason(&response);
    assert_eq!(
        aggregate_reason.code,
        SearchIndexStatusReasonCode::BuildFailed
    );
    assert_eq!(aggregate_reason.severity, SearchIndexStatusSeverity::Error);
    assert_eq!(aggregate_reason.action, SearchIndexStatusAction::RetryBuild);
    assert_eq!(aggregate_reason.affected_corpus_count, 3);
    assert_eq!(aggregate_reason.readable_corpus_count, 2);
    assert_eq!(aggregate_reason.blocking_corpus_count, 1);
}

#[test]
fn response_status_reason_prefers_compacting_over_compaction_pending() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: true,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 4,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };
    local_symbol.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::Compacting,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: true,
    });

    let mut knowledge = SearchCorpusStatus::new(SearchCorpusKind::KnowledgeSection);
    knowledge.phase = SearchPlanePhase::Ready;
    knowledge.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 1,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };
    knowledge.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::CompactionPending,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: true,
    });

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, knowledge],
    });

    let aggregate_reason = status_reason(&response);
    assert_eq!(
        aggregate_reason.code,
        SearchIndexStatusReasonCode::Compacting
    );
    assert_eq!(aggregate_reason.severity, SearchIndexStatusSeverity::Info);
    assert_eq!(aggregate_reason.action, SearchIndexStatusAction::Wait);
    assert_eq!(aggregate_reason.affected_corpus_count, 2);
    assert_eq!(aggregate_reason.readable_corpus_count, 2);
    assert_eq!(aggregate_reason.blocking_corpus_count, 0);
    assert!(response.maintenance_summary.is_some());
    assert!(response.query_telemetry_summary.is_none());
}

#[test]
fn response_status_reason_prefers_warming_up_over_prewarming() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Indexing;
    local_symbol.staging_epoch = Some(5);
    local_symbol.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::WarmingUp,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: false,
    });

    let mut knowledge = SearchCorpusStatus::new(SearchCorpusKind::KnowledgeSection);
    knowledge.phase = SearchPlanePhase::Indexing;
    knowledge.staging_epoch = Some(7);
    knowledge.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: false,
        publish_count_since_compaction: 0,
        last_prewarmed_at: Some("2026-03-24T12:34:56Z".to_string()),
        last_prewarmed_epoch: Some(7),
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };
    knowledge.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::Prewarming,
        severity: SearchCorpusStatusSeverity::Info,
        action: SearchCorpusStatusAction::Wait,
        readable: false,
    });

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, knowledge],
    });

    let aggregate_reason = status_reason(&response);
    assert_eq!(
        aggregate_reason.code,
        SearchIndexStatusReasonCode::WarmingUp
    );
    assert_eq!(aggregate_reason.severity, SearchIndexStatusSeverity::Info);
    assert_eq!(aggregate_reason.action, SearchIndexStatusAction::Wait);
    assert_eq!(aggregate_reason.affected_corpus_count, 2);
    assert_eq!(aggregate_reason.readable_corpus_count, 0);
    assert_eq!(aggregate_reason.blocking_corpus_count, 2);
    let prewarming_reason = corpus_status_reason(&response, 1);
    assert_eq!(
        prewarming_reason.code,
        SearchIndexStatusReasonCode::Prewarming
    );
    assert_eq!(prewarming_reason.severity, SearchIndexStatusSeverity::Info);
    assert_eq!(prewarming_reason.action, SearchIndexStatusAction::Wait);
    assert!(!prewarming_reason.readable);
}
