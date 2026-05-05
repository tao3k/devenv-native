use std::collections::BTreeMap;

use super::definitions as search_index;
use super::diagnostics::summarize_status_diagnostics;

impl From<&crate::search::SearchPlaneStatusSnapshot> for search_index::SearchIndexStatusResponse {
    fn from(value: &crate::search::SearchPlaneStatusSnapshot) -> Self {
        let corpora = value
            .corpora
            .iter()
            .map(search_index::SearchCorpusIndexStatus::from)
            .collect::<Vec<_>>();
        let rollup = fallback_rollup_from_corpora(&corpora);
        let status_reason = summarize_response_status_reason(&corpora);
        let query_telemetry_summary = summarize_response_query_telemetry(&corpora);
        let repo_read_pressure = value
            .repo_read_pressure
            .as_ref()
            .map(search_index::SearchIndexRepoReadPressure::from);
        Self {
            total: rollup.total,
            idle: rollup.idle,
            indexing: rollup.indexing,
            ready: rollup.ready,
            degraded: rollup.degraded,
            failed: rollup.failed,
            compaction_pending: rollup.compaction_pending,
            status_reason,
            maintenance_summary: rollup.maintenance_summary,
            query_telemetry_summary,
            repo_read_pressure,
            corpora,
        }
    }
}

impl search_index::SearchIndexStatusResponse {
    /// Build a status response and enrich it with live diagnostics when available.
    pub async fn from_snapshot_with_diagnostics(
        value: &crate::search::SearchPlaneStatusSnapshot,
    ) -> Self {
        let corpora = value
            .corpora
            .iter()
            .map(search_index::SearchCorpusIndexStatus::from)
            .collect::<Vec<_>>();
        let (rollup, status_reason, query_telemetry_summary, repo_read_pressure) =
            match summarize_status_diagnostics(value).await {
                Ok(summary) => (
                    summary.rollup,
                    summary.status_reason,
                    summary.query_telemetry_summary,
                    summary.repo_read_pressure,
                ),
                Err(_) => (
                    fallback_rollup_from_corpora(&corpora),
                    summarize_response_status_reason(&corpora),
                    summarize_response_query_telemetry(&corpora),
                    value.repo_read_pressure.as_ref().map(From::from),
                ),
            };
        Self {
            total: rollup.total,
            idle: rollup.idle,
            indexing: rollup.indexing,
            ready: rollup.ready,
            degraded: rollup.degraded,
            failed: rollup.failed,
            compaction_pending: rollup.compaction_pending,
            status_reason,
            maintenance_summary: rollup.maintenance_summary,
            query_telemetry_summary,
            repo_read_pressure,
            corpora,
        }
    }
}

fn fallback_rollup_from_corpora(
    corpora: &[search_index::SearchCorpusIndexStatus],
) -> super::diagnostics::SearchIndexDiagnosticsRollup {
    super::diagnostics::SearchIndexDiagnosticsRollup {
        total: corpora.len(),
        idle: corpora
            .iter()
            .filter(|status| matches!(status.phase, search_index::SearchIndexPhase::Idle))
            .count(),
        indexing: corpora
            .iter()
            .filter(|status| matches!(status.phase, search_index::SearchIndexPhase::Indexing))
            .count(),
        ready: corpora
            .iter()
            .filter(|status| matches!(status.phase, search_index::SearchIndexPhase::Ready))
            .count(),
        failed: corpora
            .iter()
            .filter(|status| matches!(status.phase, search_index::SearchIndexPhase::Failed))
            .count(),
        degraded: corpora
            .iter()
            .filter(|status| matches!(status.phase, search_index::SearchIndexPhase::Degraded))
            .count(),
        compaction_pending: corpora
            .iter()
            .filter(|status| status.maintenance.compaction_pending)
            .count(),
        maintenance_summary: summarize_response_maintenance(corpora),
    }
}

fn summarize_response_status_reason(
    corpora: &[search_index::SearchCorpusIndexStatus],
) -> Option<search_index::SearchIndexAggregateStatusReason> {
    let reasons = corpora
        .iter()
        .filter_map(|status| status.status_reason.as_ref())
        .collect::<Vec<_>>();
    let primary = reasons.into_iter().min_by_key(|reason| {
        (
            response_reason_severity_priority(reason.severity),
            response_reason_code_priority(reason.code),
        )
    })?;
    let affected_corpus_count = corpora
        .iter()
        .filter(|status| status.status_reason.is_some())
        .count();
    let readable_corpus_count = corpora
        .iter()
        .filter_map(|status| status.status_reason.as_ref())
        .filter(|reason| reason.readable)
        .count();
    let blocking_corpus_count = affected_corpus_count.saturating_sub(readable_corpus_count);
    Some(search_index::SearchIndexAggregateStatusReason {
        code: primary.code,
        severity: primary.severity,
        action: primary.action,
        affected_corpus_count,
        readable_corpus_count,
        blocking_corpus_count,
    })
}

fn summarize_response_query_telemetry(
    corpora: &[search_index::SearchCorpusIndexStatus],
) -> Option<search_index::SearchIndexAggregateQueryTelemetry> {
    let telemetry = corpora
        .iter()
        .filter_map(|status| status.last_query_telemetry.as_ref())
        .collect::<Vec<_>>();
    if telemetry.is_empty() {
        return None;
    }

    let mut summary = QueryTelemetryAccumulator::default();
    let mut scopes = BTreeMap::<String, QueryTelemetryAccumulator>::new();

    for entry in telemetry {
        summary.observe(entry);
        if let Some(scope) = entry.scope.as_deref().filter(|scope| !scope.is_empty()) {
            scopes.entry(scope.to_string()).or_default().observe(entry);
        }
    }

    Some(
        summary.into_aggregate(
            scopes
                .into_iter()
                .map(|(scope, bucket)| bucket.into_scope_summary(scope))
                .collect(),
        ),
    )
}

fn summarize_response_maintenance(
    corpora: &[search_index::SearchCorpusIndexStatus],
) -> Option<search_index::SearchIndexAggregateMaintenanceSummary> {
    let prewarm_running_count = corpora
        .iter()
        .filter(|status| status.maintenance.prewarm_running)
        .count();
    let prewarm_queued_corpus_count = corpora
        .iter()
        .filter(|status| status.maintenance.prewarm_queue_depth > 0)
        .count();
    let max_prewarm_queue_depth = corpora
        .iter()
        .map(|status| status.maintenance.prewarm_queue_depth)
        .max()
        .unwrap_or_default();
    let compaction_running_count = corpora
        .iter()
        .filter(|status| status.maintenance.compaction_running)
        .count();
    let compaction_queued_corpus_count = corpora
        .iter()
        .filter(|status| status.maintenance.compaction_queue_depth > 0)
        .count();
    let max_compaction_queue_depth = corpora
        .iter()
        .map(|status| status.maintenance.compaction_queue_depth)
        .max()
        .unwrap_or_default();
    let compaction_pending_count = corpora
        .iter()
        .filter(|status| status.maintenance.compaction_pending)
        .count();
    let aged_compaction_queue_count = corpora
        .iter()
        .filter(|status| status.maintenance.compaction_queue_aged.is_aged())
        .count();

    if prewarm_running_count == 0
        && prewarm_queued_corpus_count == 0
        && compaction_running_count == 0
        && compaction_queued_corpus_count == 0
        && compaction_pending_count == 0
        && aged_compaction_queue_count == 0
    {
        return None;
    }

    Some(search_index::SearchIndexAggregateMaintenanceSummary {
        prewarm_running_count,
        prewarm_queued_corpus_count,
        max_prewarm_queue_depth,
        compaction_running_count,
        compaction_queued_corpus_count,
        max_compaction_queue_depth,
        compaction_pending_count,
        aged_compaction_queue_count,
    })
}

fn max_optional_u64(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

pub(super) fn response_reason_severity_priority(
    severity: search_index::SearchIndexStatusSeverity,
) -> u8 {
    match severity {
        search_index::SearchIndexStatusSeverity::Error => 0,
        search_index::SearchIndexStatusSeverity::Warning => 1,
        search_index::SearchIndexStatusSeverity::Info => 2,
    }
}

pub(super) fn response_reason_code_priority(code: search_index::SearchIndexStatusReasonCode) -> u8 {
    match code {
        search_index::SearchIndexStatusReasonCode::PublishedManifestMissing => 0,
        search_index::SearchIndexStatusReasonCode::BuildFailed => 1,
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMissing => 2,
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMismatch => 3,
        search_index::SearchIndexStatusReasonCode::RepoIndexFailed => 4,
        search_index::SearchIndexStatusReasonCode::WarmingUp => 5,
        search_index::SearchIndexStatusReasonCode::Prewarming => 6,
        search_index::SearchIndexStatusReasonCode::Refreshing => 7,
        search_index::SearchIndexStatusReasonCode::Compacting => 8,
        search_index::SearchIndexStatusReasonCode::CompactionPending => 9,
    }
}

#[derive(Debug, Default)]
struct QueryTelemetryAccumulator {
    corpus_count: usize,
    latest_captured_at: String,
    scan_count: usize,
    fts_count: usize,
    fts_fallback_scan_count: usize,
    total_rows_scanned: u64,
    total_matched_rows: u64,
    total_result_count: u64,
    max_batch_row_limit: Option<u64>,
    max_recall_limit_rows: Option<u64>,
    max_working_set_budget_rows: u64,
    max_trim_threshold_rows: u64,
    max_peak_working_set_rows: u64,
    total_trim_count: u64,
    total_dropped_candidate_count: u64,
}

impl QueryTelemetryAccumulator {
    fn observe(&mut self, entry: &search_index::SearchIndexQueryTelemetry) {
        self.corpus_count = self.corpus_count.saturating_add(1);
        if self.latest_captured_at.as_str() < entry.captured_at.as_str() {
            self.latest_captured_at.clone_from(&entry.captured_at);
        }
        match entry.source {
            search_index::SearchIndexQueryTelemetrySource::Scan => {
                self.scan_count = self.scan_count.saturating_add(1);
            }
            search_index::SearchIndexQueryTelemetrySource::Fts => {
                self.fts_count = self.fts_count.saturating_add(1);
            }
            search_index::SearchIndexQueryTelemetrySource::FtsFallbackScan => {
                self.fts_fallback_scan_count = self.fts_fallback_scan_count.saturating_add(1);
            }
        }
        self.total_rows_scanned = self.total_rows_scanned.saturating_add(entry.rows_scanned);
        self.total_matched_rows = self.total_matched_rows.saturating_add(entry.matched_rows);
        self.total_result_count = self.total_result_count.saturating_add(entry.result_count);
        self.max_batch_row_limit =
            max_optional_u64(self.max_batch_row_limit, entry.batch_row_limit);
        self.max_recall_limit_rows =
            max_optional_u64(self.max_recall_limit_rows, entry.recall_limit_rows);
        self.max_working_set_budget_rows = self
            .max_working_set_budget_rows
            .max(entry.working_set_budget_rows);
        self.max_trim_threshold_rows = self.max_trim_threshold_rows.max(entry.trim_threshold_rows);
        self.max_peak_working_set_rows = self
            .max_peak_working_set_rows
            .max(entry.peak_working_set_rows);
        self.total_trim_count = self.total_trim_count.saturating_add(entry.trim_count);
        self.total_dropped_candidate_count = self
            .total_dropped_candidate_count
            .saturating_add(entry.dropped_candidate_count);
    }

    fn into_aggregate(
        self,
        scopes: Vec<search_index::SearchIndexQueryTelemetryScopeSummary>,
    ) -> search_index::SearchIndexAggregateQueryTelemetry {
        search_index::SearchIndexAggregateQueryTelemetry {
            corpus_count: self.corpus_count,
            latest_captured_at: self.latest_captured_at,
            scan_count: self.scan_count,
            fts_count: self.fts_count,
            fts_fallback_scan_count: self.fts_fallback_scan_count,
            total_rows_scanned: self.total_rows_scanned,
            total_matched_rows: self.total_matched_rows,
            total_result_count: self.total_result_count,
            max_batch_row_limit: self.max_batch_row_limit,
            max_recall_limit_rows: self.max_recall_limit_rows,
            max_working_set_budget_rows: self.max_working_set_budget_rows,
            max_trim_threshold_rows: self.max_trim_threshold_rows,
            max_peak_working_set_rows: self.max_peak_working_set_rows,
            total_trim_count: self.total_trim_count,
            total_dropped_candidate_count: self.total_dropped_candidate_count,
            scopes,
        }
    }

    fn into_scope_summary(
        self,
        scope: String,
    ) -> search_index::SearchIndexQueryTelemetryScopeSummary {
        search_index::SearchIndexQueryTelemetryScopeSummary {
            scope,
            corpus_count: self.corpus_count,
            latest_captured_at: self.latest_captured_at,
            scan_count: self.scan_count,
            fts_count: self.fts_count,
            fts_fallback_scan_count: self.fts_fallback_scan_count,
            total_rows_scanned: self.total_rows_scanned,
            total_matched_rows: self.total_matched_rows,
            total_result_count: self.total_result_count,
            max_batch_row_limit: self.max_batch_row_limit,
            max_recall_limit_rows: self.max_recall_limit_rows,
            max_working_set_budget_rows: self.max_working_set_budget_rows,
            max_trim_threshold_rows: self.max_trim_threshold_rows,
            max_peak_working_set_rows: self.max_peak_working_set_rows,
            total_trim_count: self.total_trim_count,
            total_dropped_candidate_count: self.total_dropped_candidate_count,
        }
    }
}
