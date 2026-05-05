use crate::search::contracts::search_index::{
    SearchIndexQueryTelemetrySource, SearchIndexStatusResponse,
};
use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchMaintenanceStatus, SearchPlanePhase,
    SearchPlaneStatusSnapshot, SearchQueryTelemetry, SearchQueryTelemetrySource,
};

use super::helpers::telemetry_attachment_status;

#[test]
fn response_maintenance_summary_rolls_up_queue_and_aging_state() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.maintenance = SearchMaintenanceStatus {
        prewarm_running: false,
        prewarm_queue_depth: 0,
        prewarm_queue_position: None,
        compaction_running: false,
        compaction_queue_depth: 2,
        compaction_queue_position: Some(2),
        compaction_queue_aged: true.into(),
        compaction_pending: true,
        publish_count_since_compaction: 3,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };

    let mut repo_entity = SearchCorpusStatus::new(SearchCorpusKind::RepoEntity);
    repo_entity.phase = SearchPlanePhase::Indexing;
    repo_entity.maintenance = SearchMaintenanceStatus {
        prewarm_running: true,
        prewarm_queue_depth: 1,
        prewarm_queue_position: Some(1),
        compaction_running: true,
        compaction_queue_depth: 1,
        compaction_queue_position: Some(1),
        compaction_queue_aged: false.into(),
        compaction_pending: true,
        publish_count_since_compaction: 1,
        last_prewarmed_at: None,
        last_prewarmed_epoch: None,
        last_compacted_at: None,
        last_compaction_reason: None,
        last_compacted_row_count: None,
    };

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, repo_entity],
    });

    let summary = response
        .maintenance_summary
        .as_ref()
        .unwrap_or_else(|| panic!("maintenance summary should be present"));
    assert_eq!(summary.prewarm_running_count, 1);
    assert_eq!(summary.prewarm_queued_corpus_count, 1);
    assert_eq!(summary.max_prewarm_queue_depth, 1);
    assert_eq!(summary.compaction_running_count, 1);
    assert_eq!(summary.compaction_queued_corpus_count, 2);
    assert_eq!(summary.max_compaction_queue_depth, 2);
    assert_eq!(summary.compaction_pending_count, 2);
    assert_eq!(summary.aged_compaction_queue_count, 1);
}

#[test]
fn response_maintenance_summary_stays_empty_without_signals() {
    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![
            SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol),
            SearchCorpusStatus::new(SearchCorpusKind::Attachment),
        ],
    });

    assert!(response.maintenance_summary.is_none());
}

#[test]
fn response_query_telemetry_summary_remains_empty_without_corpus_telemetry() {
    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![
            SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol),
            SearchCorpusStatus::new(SearchCorpusKind::Attachment),
        ],
    });

    assert!(response.query_telemetry_summary.is_none());
}

#[test]
fn response_query_telemetry_summary_preserves_source_mapping() {
    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![telemetry_attachment_status()],
    });

    let summary = response
        .query_telemetry_summary
        .as_ref()
        .unwrap_or_else(|| panic!("query telemetry summary should be present"));
    assert_eq!(summary.scan_count, 0);
    assert_eq!(summary.fts_count, 0);
    assert_eq!(summary.fts_fallback_scan_count, 1);
    assert_eq!(summary.scopes.len(), 1);
    assert_eq!(summary.scopes[0].scope, "alpha/repo");
    assert_eq!(summary.scopes[0].fts_fallback_scan_count, 1);
    assert_eq!(
        response.corpora[0]
            .last_query_telemetry
            .as_ref()
            .map(|telemetry| telemetry.source),
        Some(SearchIndexQueryTelemetrySource::FtsFallbackScan)
    );
}

#[test]
fn response_query_telemetry_summary_groups_rows_by_scope_hint() {
    let mut local_symbol = SearchCorpusStatus::new(SearchCorpusKind::LocalSymbol);
    local_symbol.phase = SearchPlanePhase::Ready;
    local_symbol.last_query_telemetry = Some(SearchQueryTelemetry {
        captured_at: "2026-03-23T22:10:00Z".to_string(),
        scope: Some("autocomplete".to_string()),
        source: SearchQueryTelemetrySource::Scan,
        batch_count: 2,
        rows_scanned: 25,
        matched_rows: 9,
        result_count: 5,
        batch_row_limit: Some(16),
        recall_limit_rows: Some(32),
        working_set_budget_rows: 12,
        trim_threshold_rows: 24,
        peak_working_set_rows: 14,
        trim_count: 1,
        dropped_candidate_count: 3,
    });

    let mut reference = SearchCorpusStatus::new(SearchCorpusKind::ReferenceOccurrence);
    reference.phase = SearchPlanePhase::Ready;
    reference.last_query_telemetry = Some(SearchQueryTelemetry {
        captured_at: "2026-03-23T22:11:00Z".to_string(),
        scope: Some("search".to_string()),
        source: SearchQueryTelemetrySource::Fts,
        batch_count: 3,
        rows_scanned: 40,
        matched_rows: 12,
        result_count: 6,
        batch_row_limit: Some(24),
        recall_limit_rows: Some(48),
        working_set_budget_rows: 18,
        trim_threshold_rows: 36,
        peak_working_set_rows: 21,
        trim_count: 0,
        dropped_candidate_count: 0,
    });

    let mut attachment = SearchCorpusStatus::new(SearchCorpusKind::Attachment);
    attachment.phase = SearchPlanePhase::Ready;
    attachment.last_query_telemetry = Some(SearchQueryTelemetry {
        captured_at: "2026-03-23T22:12:00Z".to_string(),
        scope: Some("search".to_string()),
        source: SearchQueryTelemetrySource::FtsFallbackScan,
        batch_count: 4,
        rows_scanned: 60,
        matched_rows: 15,
        result_count: 7,
        batch_row_limit: Some(32),
        recall_limit_rows: Some(64),
        working_set_budget_rows: 24,
        trim_threshold_rows: 48,
        peak_working_set_rows: 29,
        trim_count: 2,
        dropped_candidate_count: 5,
    });

    let response = SearchIndexStatusResponse::from(&SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora: vec![local_symbol, reference, attachment],
    });

    let summary = response
        .query_telemetry_summary
        .as_ref()
        .unwrap_or_else(|| panic!("query telemetry summary should be present"));
    assert_eq!(summary.corpus_count, 3);
    assert_eq!(summary.scopes.len(), 2);
    assert_eq!(summary.scopes[0].scope, "autocomplete");
    assert_eq!(summary.scopes[0].corpus_count, 1);
    assert_eq!(summary.scopes[0].scan_count, 1);
    assert_eq!(summary.scopes[0].fts_count, 0);
    assert_eq!(summary.scopes[0].fts_fallback_scan_count, 0);
    assert_eq!(summary.scopes[0].total_rows_scanned, 25);
    assert_eq!(summary.scopes[1].scope, "search");
    assert_eq!(summary.scopes[1].corpus_count, 2);
    assert_eq!(summary.scopes[1].scan_count, 0);
    assert_eq!(summary.scopes[1].fts_count, 1);
    assert_eq!(summary.scopes[1].fts_fallback_scan_count, 1);
    assert_eq!(summary.scopes[1].total_rows_scanned, 100);
    assert_eq!(summary.scopes[1].total_matched_rows, 27);
    assert_eq!(summary.scopes[1].total_result_count, 13);
    assert_eq!(summary.scopes[1].max_batch_row_limit, Some(32));
    assert_eq!(summary.scopes[1].max_recall_limit_rows, Some(64));
    assert_eq!(summary.scopes[1].total_trim_count, 2);
    assert_eq!(summary.scopes[1].total_dropped_candidate_count, 5);
}
