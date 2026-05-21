//! `search::status::telemetry` owns Wendao search status telemetry behavior.

use serde::{Deserialize, Serialize};

/// Source path used by the most recent bounded streaming query for one corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchQueryTelemetrySource {
    /// The query streamed batches from a regular projected scan only.
    Scan,
    /// The query streamed batches from FTS only.
    Fts,
    /// The query attempted FTS first and then fell back to a regular projected scan.
    FtsFallbackScan,
}

/// Recent bounded-rerank telemetry recorded for one corpus query lane.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQueryTelemetry {
    /// RFC3339 timestamp when the telemetry record was captured.
    pub captured_at: String,
    /// Optional scope hint such as a repo identifier for repo-backed queries.
    pub scope: Option<String>,
    /// Streaming source used by the query.
    pub source: SearchQueryTelemetrySource,
    /// Number of streamed batches consumed by the query.
    pub batch_count: u64,
    /// Total number of rows scanned across all streamed batches.
    pub rows_scanned: u64,
    /// Number of rows that matched the lexical predicate before bounded trimming.
    pub matched_rows: u64,
    /// Final number of retained results returned to the caller.
    pub result_count: u64,
    /// Batch row limit used for projected scan requests, when configured.
    pub batch_row_limit: Option<u64>,
    /// Recall limit pushed into the Lance scan/FTS layer, when configured.
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
