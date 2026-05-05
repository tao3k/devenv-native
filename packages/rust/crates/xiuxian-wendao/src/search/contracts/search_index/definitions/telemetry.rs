use serde::{Deserialize, Serialize};
use specta::Type;

/// Source path used by the most recent bounded streaming query for one corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SearchIndexQueryTelemetrySource {
    /// The query streamed batches from a regular projected scan only.
    Scan,
    /// The query streamed batches from FTS only.
    Fts,
    /// The query attempted FTS first and then fell back to a regular projected scan.
    FtsFallbackScan,
}

/// Recent bounded-rerank telemetry recorded for one corpus query lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexQueryTelemetry {
    /// RFC3339 timestamp when the telemetry record was captured.
    pub captured_at: String,
    /// Optional scope hint such as a repo identifier for repo-backed queries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Streaming source used by the query.
    pub source: SearchIndexQueryTelemetrySource,
    /// Number of streamed batches consumed by the query.
    pub batch_count: u64,
    /// Total number of rows scanned across all streamed batches.
    pub rows_scanned: u64,
    /// Number of rows that matched the lexical predicate before bounded trimming.
    pub matched_rows: u64,
    /// Final number of retained results returned to the caller.
    pub result_count: u64,
    /// Batch row limit used for projected scan requests, when configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch_row_limit: Option<u64>,
    /// Recall limit pushed into the Lance scan/FTS layer, when configured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recall_limit_rows: Option<u64>,
    /// Soft in-memory working-set budget expressed as retained candidate rows.
    pub working_set_budget_rows: u64,
    /// Trim threshold that triggers bounded compaction of the working set.
    pub trim_threshold_rows: u64,
    /// Largest candidate/path working set observed during the query.
    pub peak_working_set_rows: u64,
    /// Number of times the working set had to be trimmed.
    pub trim_count: u64,
    /// Number of candidates/paths dropped by bounded trimming.
    pub dropped_candidate_count: u64,
}

/// Response-level summary derived from the most recent per-corpus query telemetry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexAggregateQueryTelemetry {
    /// Number of corpora contributing recent query telemetry.
    pub corpus_count: usize,
    /// RFC3339 timestamp of the most recent telemetry record in the response.
    pub latest_captured_at: String,
    /// Number of corpora whose most recent query used a projected scan only.
    pub scan_count: usize,
    /// Number of corpora whose most recent query used FTS only.
    pub fts_count: usize,
    /// Number of corpora whose most recent query fell back from FTS to projected scan.
    pub fts_fallback_scan_count: usize,
    /// Total rows scanned across the retained telemetry set.
    pub total_rows_scanned: u64,
    /// Total lexical matches observed before bounded trimming.
    pub total_matched_rows: u64,
    /// Total retained results returned by the recorded queries.
    pub total_result_count: u64,
    /// Maximum batch row limit observed across the retained telemetry set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_batch_row_limit: Option<u64>,
    /// Maximum recall limit observed across the retained telemetry set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_recall_limit_rows: Option<u64>,
    /// Largest working-set budget observed across the retained telemetry set.
    pub max_working_set_budget_rows: u64,
    /// Largest trim threshold observed across the retained telemetry set.
    pub max_trim_threshold_rows: u64,
    /// Largest observed peak working set across the retained telemetry set.
    pub max_peak_working_set_rows: u64,
    /// Total number of trim events observed across the retained telemetry set.
    pub total_trim_count: u64,
    /// Total number of dropped candidates/paths observed across the retained telemetry set.
    pub total_dropped_candidate_count: u64,
    /// Per-scope rollups for telemetry rows carrying a non-empty scope hint.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<SearchIndexQueryTelemetryScopeSummary>,
}

/// Response-level telemetry rollup for one concrete scope hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexQueryTelemetryScopeSummary {
    /// Opaque scope hint observed on the contributing telemetry rows.
    pub scope: String,
    /// Number of corpora contributing recent query telemetry for this scope.
    pub corpus_count: usize,
    /// RFC3339 timestamp of the most recent telemetry record in this scope bucket.
    pub latest_captured_at: String,
    /// Number of corpora in this scope bucket whose most recent query used a projected scan only.
    pub scan_count: usize,
    /// Number of corpora in this scope bucket whose most recent query used FTS only.
    pub fts_count: usize,
    /// Number of corpora in this scope bucket whose most recent query fell back from FTS to projected scan.
    pub fts_fallback_scan_count: usize,
    /// Total rows scanned across the retained telemetry set for this scope bucket.
    pub total_rows_scanned: u64,
    /// Total lexical matches observed before bounded trimming for this scope bucket.
    pub total_matched_rows: u64,
    /// Total retained results returned by the recorded queries for this scope bucket.
    pub total_result_count: u64,
    /// Maximum batch row limit observed across this scope bucket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_batch_row_limit: Option<u64>,
    /// Maximum recall limit observed across this scope bucket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_recall_limit_rows: Option<u64>,
    /// Largest working-set budget observed across this scope bucket.
    pub max_working_set_budget_rows: u64,
    /// Largest trim threshold observed across this scope bucket.
    pub max_trim_threshold_rows: u64,
    /// Largest observed peak working set across this scope bucket.
    pub max_peak_working_set_rows: u64,
    /// Total number of trim events observed across this scope bucket.
    pub total_trim_count: u64,
    /// Total number of dropped candidates/paths observed across this scope bucket.
    pub total_dropped_candidate_count: u64,
}
