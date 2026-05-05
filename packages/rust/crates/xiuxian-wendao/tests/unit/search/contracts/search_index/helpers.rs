use crate::search::contracts::search_index::{
    SearchIndexAggregateStatusReason, SearchIndexIssueSummary, SearchIndexStatusReason,
    SearchIndexStatusResponse,
};
use crate::search::{
    SearchCorpusIssue, SearchCorpusIssueCode, SearchCorpusIssueFamily, SearchCorpusKind,
    SearchCorpusStatus, SearchCorpusStatusAction, SearchCorpusStatusReason,
    SearchCorpusStatusReasonCode, SearchCorpusStatusSeverity, SearchMaintenanceStatus,
    SearchPlanePhase, SearchQueryTelemetry, SearchQueryTelemetrySource,
};

pub(super) fn status_reason(
    response: &SearchIndexStatusResponse,
) -> &SearchIndexAggregateStatusReason {
    response
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("aggregate status reason should be present"))
}

pub(super) fn corpus_status_reason(
    response: &SearchIndexStatusResponse,
    index: usize,
) -> &SearchIndexStatusReason {
    response.corpora[index]
        .status_reason
        .as_ref()
        .unwrap_or_else(|| panic!("status reason should be present"))
}

pub(super) fn corpus_issue_summary(
    response: &SearchIndexStatusResponse,
    index: usize,
) -> &SearchIndexIssueSummary {
    response.corpora[index]
        .issue_summary
        .as_ref()
        .unwrap_or_else(|| panic!("issue summary should be present"))
}

pub(super) fn compacting_local_symbol_status() -> SearchCorpusStatus {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.active_epoch = Some(3);
    local_symbol.row_count = Some(10);
    local_symbol.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: true,
        compaction_queue_depth: 0,
        compaction_queue_position: None,
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 3,
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
    local_symbol
}

pub(super) fn degraded_repo_entity_status() -> SearchCorpusStatus {
    let mut repo_entity = SearchCorpusStatus::new(SearchCorpusKind::RepoEntity);
    repo_entity.phase = SearchPlanePhase::Degraded;
    repo_entity.issues.push(SearchCorpusIssue {
        code: SearchCorpusIssueCode::PublishedRevisionMismatch,
        readable: true,
        repo_id: Some("alpha/repo".to_string()),
        current_revision: Some("rev-2".to_string()),
        published_revision: Some("rev-1".to_string()),
        message: "alpha/repo drifted".to_string(),
    });
    repo_entity.issue_summary = Some(crate::search::SearchCorpusIssueSummary {
        family: SearchCorpusIssueFamily::Revision,
        primary_code: SearchCorpusIssueCode::PublishedRevisionMismatch,
        issue_count: 1,
        readable_issue_count: 1,
    });
    repo_entity.status_reason = Some(SearchCorpusStatusReason {
        code: SearchCorpusStatusReasonCode::PublishedRevisionMismatch,
        severity: SearchCorpusStatusSeverity::Warning,
        action: SearchCorpusStatusAction::ResyncRepo,
        readable: true,
    });
    repo_entity
}

pub(super) fn telemetry_attachment_status() -> SearchCorpusStatus {
    let mut attachment = SearchCorpusStatus::new(SearchCorpusKind::Attachment);
    attachment.phase = SearchPlanePhase::Ready;
    attachment.active_epoch = Some(9);
    attachment.last_query_telemetry = Some(SearchQueryTelemetry {
        captured_at: "2026-03-23T22:05:00Z".to_string(),
        scope: Some("alpha/repo".to_string()),
        source: SearchQueryTelemetrySource::FtsFallbackScan,
        batch_count: 4,
        rows_scanned: 96,
        matched_rows: 19,
        result_count: 8,
        batch_row_limit: Some(32),
        recall_limit_rows: Some(64),
        working_set_budget_rows: 24,
        trim_threshold_rows: 48,
        peak_working_set_rows: 41,
        trim_count: 2,
        dropped_candidate_count: 11,
    });
    attachment
}

pub(super) fn telemetry_knowledge_status() -> SearchCorpusStatus {
    let mut knowledge = SearchCorpusStatus::new(SearchCorpusKind::KnowledgeSection);
    knowledge.phase = SearchPlanePhase::Ready;
    knowledge.active_epoch = Some(4);
    knowledge.last_query_telemetry = Some(SearchQueryTelemetry {
        captured_at: "2026-03-23T22:07:00Z".to_string(),
        scope: None,
        source: SearchQueryTelemetrySource::Fts,
        batch_count: 2,
        rows_scanned: 70,
        matched_rows: 14,
        result_count: 6,
        batch_row_limit: Some(16),
        recall_limit_rows: Some(40),
        working_set_budget_rows: 12,
        trim_threshold_rows: 24,
        peak_working_set_rows: 18,
        trim_count: 1,
        dropped_candidate_count: 5,
    });
    knowledge
}
