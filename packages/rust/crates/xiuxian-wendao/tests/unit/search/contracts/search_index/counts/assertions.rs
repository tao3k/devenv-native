use crate::search::contracts::search_index::{
    SearchIndexIssueCode, SearchIndexIssueFamily, SearchIndexPhase,
    SearchIndexQueryTelemetrySource, SearchIndexStatusAction, SearchIndexStatusReasonCode,
    SearchIndexStatusResponse, SearchIndexStatusSeverity,
};

use crate::search::contracts::search_index::tests::helpers::{
    corpus_issue_summary, corpus_status_reason, status_reason,
};

pub(super) fn assert_response_counts(response: &SearchIndexStatusResponse) {
    assert_eq!(response.total, 4);
    assert_eq!(response.idle, 0);
    assert_eq!(response.indexing, 0);
    assert_eq!(response.ready, 3);
    assert_eq!(response.degraded, 1);
    assert_eq!(response.failed, 0);
    assert_eq!(response.compaction_pending, 1);
}

pub(super) fn assert_aggregate_reason(response: &SearchIndexStatusResponse) {
    let aggregate_reason = status_reason(response);
    assert_eq!(
        aggregate_reason.code,
        SearchIndexStatusReasonCode::PublishedRevisionMismatch
    );
    assert_eq!(
        aggregate_reason.severity,
        SearchIndexStatusSeverity::Warning
    );
    assert_eq!(aggregate_reason.action, SearchIndexStatusAction::ResyncRepo);
    assert_eq!(aggregate_reason.affected_corpus_count, 2);
    assert_eq!(aggregate_reason.readable_corpus_count, 2);
    assert_eq!(aggregate_reason.blocking_corpus_count, 0);
}

pub(super) fn assert_maintenance_summary(response: &SearchIndexStatusResponse) {
    let Some(maintenance_summary) = response.maintenance_summary.as_ref() else {
        panic!("maintenance summary should be present");
    };
    assert_eq!(maintenance_summary.prewarm_running_count, 0);
    assert_eq!(maintenance_summary.prewarm_queued_corpus_count, 0);
    assert_eq!(maintenance_summary.max_prewarm_queue_depth, 0);
    assert_eq!(maintenance_summary.compaction_running_count, 1);
    assert_eq!(maintenance_summary.compaction_queued_corpus_count, 0);
    assert_eq!(maintenance_summary.max_compaction_queue_depth, 0);
    assert_eq!(maintenance_summary.compaction_pending_count, 1);
    assert_eq!(maintenance_summary.aged_compaction_queue_count, 0);
}

pub(super) fn assert_local_compaction_status(response: &SearchIndexStatusResponse) {
    assert_eq!(response.corpora[0].phase, SearchIndexPhase::Ready);
    let local_reason = corpus_status_reason(response, 0);
    assert_eq!(local_reason.code, SearchIndexStatusReasonCode::Compacting);
    assert_eq!(local_reason.severity, SearchIndexStatusSeverity::Info);
    assert_eq!(local_reason.action, SearchIndexStatusAction::Wait);
    assert!(local_reason.readable);
    assert!(response.corpora[0].maintenance.compaction_running);
}

pub(super) fn assert_repo_issue_rollup(response: &SearchIndexStatusResponse) {
    assert_eq!(response.corpora[1].issues.len(), 1);
    assert_eq!(
        response.corpora[1].issues[0].code,
        SearchIndexIssueCode::PublishedRevisionMismatch
    );
    let summary = corpus_issue_summary(response, 1);
    assert_eq!(summary.family, SearchIndexIssueFamily::Revision);
    assert_eq!(
        summary.primary_code,
        SearchIndexIssueCode::PublishedRevisionMismatch
    );
    assert_eq!(summary.issue_count, 1);
    assert_eq!(summary.readable_issue_count, 1);
    let reason = corpus_status_reason(response, 1);
    assert_eq!(
        reason.code,
        SearchIndexStatusReasonCode::PublishedRevisionMismatch
    );
    assert_eq!(reason.severity, SearchIndexStatusSeverity::Warning);
    assert_eq!(reason.action, SearchIndexStatusAction::ResyncRepo);
    assert!(reason.readable);
}

pub(super) fn assert_attachment_telemetry(response: &SearchIndexStatusResponse) {
    let Some(telemetry) = response.corpora[2].last_query_telemetry.as_ref() else {
        panic!("telemetry should be present");
    };
    assert_eq!(
        telemetry.source,
        SearchIndexQueryTelemetrySource::FtsFallbackScan
    );
    assert_eq!(telemetry.scope.as_deref(), Some("alpha/repo"));
    assert_eq!(telemetry.batch_count, 4);
    assert_eq!(telemetry.rows_scanned, 96);
    assert_eq!(telemetry.matched_rows, 19);
    assert_eq!(telemetry.result_count, 8);
    assert_eq!(telemetry.batch_row_limit, Some(32));
    assert_eq!(telemetry.recall_limit_rows, Some(64));
    assert_eq!(telemetry.working_set_budget_rows, 24);
    assert_eq!(telemetry.trim_threshold_rows, 48);
    assert_eq!(telemetry.peak_working_set_rows, 41);
    assert_eq!(telemetry.trim_count, 2);
    assert_eq!(telemetry.dropped_candidate_count, 11);
}

pub(super) fn assert_telemetry_summary(response: &SearchIndexStatusResponse) {
    let Some(telemetry_summary) = response.query_telemetry_summary.as_ref() else {
        panic!("query telemetry summary should be present");
    };
    assert_eq!(telemetry_summary.corpus_count, 2);
    assert_eq!(telemetry_summary.latest_captured_at, "2026-03-23T22:07:00Z");
    assert_eq!(telemetry_summary.scan_count, 0);
    assert_eq!(telemetry_summary.fts_count, 1);
    assert_eq!(telemetry_summary.fts_fallback_scan_count, 1);
    assert_eq!(telemetry_summary.total_rows_scanned, 166);
    assert_eq!(telemetry_summary.total_matched_rows, 33);
    assert_eq!(telemetry_summary.total_result_count, 14);
    assert_eq!(telemetry_summary.max_batch_row_limit, Some(32));
    assert_eq!(telemetry_summary.max_recall_limit_rows, Some(64));
    assert_eq!(telemetry_summary.max_working_set_budget_rows, 24);
    assert_eq!(telemetry_summary.max_trim_threshold_rows, 48);
    assert_eq!(telemetry_summary.max_peak_working_set_rows, 41);
    assert_eq!(telemetry_summary.total_trim_count, 3);
    assert_eq!(telemetry_summary.total_dropped_candidate_count, 16);
    assert_eq!(telemetry_summary.scopes.len(), 1);
    assert_eq!(telemetry_summary.scopes[0].scope, "alpha/repo");
    assert_eq!(telemetry_summary.scopes[0].corpus_count, 1);
    assert_eq!(
        telemetry_summary.scopes[0].latest_captured_at,
        "2026-03-23T22:05:00Z"
    );
    assert_eq!(telemetry_summary.scopes[0].scan_count, 0);
    assert_eq!(telemetry_summary.scopes[0].fts_count, 0);
    assert_eq!(telemetry_summary.scopes[0].fts_fallback_scan_count, 1);
    assert_eq!(telemetry_summary.scopes[0].total_rows_scanned, 96);
}
