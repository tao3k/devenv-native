pub(super) const STATUS_DIAGNOSTICS_SQL: &str = r"
SELECT
    CAST(COUNT(*) AS BIGINT) AS total,
    CAST(SUM(CASE WHEN phase = 'idle' THEN 1 ELSE 0 END) AS BIGINT) AS idle,
    CAST(SUM(CASE WHEN phase = 'indexing' THEN 1 ELSE 0 END) AS BIGINT) AS indexing,
    CAST(SUM(CASE WHEN phase = 'ready' THEN 1 ELSE 0 END) AS BIGINT) AS ready,
    CAST(SUM(CASE WHEN phase = 'degraded' THEN 1 ELSE 0 END) AS BIGINT) AS degraded,
    CAST(SUM(CASE WHEN phase = 'failed' THEN 1 ELSE 0 END) AS BIGINT) AS failed,
    CAST(SUM(CASE WHEN compaction_pending THEN 1 ELSE 0 END) AS BIGINT) AS compaction_pending,
    CAST(SUM(CASE WHEN prewarm_running THEN 1 ELSE 0 END) AS BIGINT) AS prewarm_running_count,
    CAST(SUM(CASE WHEN prewarm_queue_depth > 0 THEN 1 ELSE 0 END) AS BIGINT) AS prewarm_queued_corpus_count,
    CAST(COALESCE(MAX(prewarm_queue_depth), 0) AS BIGINT) AS max_prewarm_queue_depth,
    CAST(SUM(CASE WHEN compaction_running THEN 1 ELSE 0 END) AS BIGINT) AS compaction_running_count,
    CAST(SUM(CASE WHEN compaction_queue_depth > 0 THEN 1 ELSE 0 END) AS BIGINT) AS compaction_queued_corpus_count,
    CAST(COALESCE(MAX(compaction_queue_depth), 0) AS BIGINT) AS max_compaction_queue_depth,
    CAST(SUM(CASE WHEN compaction_pending THEN 1 ELSE 0 END) AS BIGINT) AS compaction_pending_count,
    CAST(SUM(CASE WHEN compaction_queue_aged THEN 1 ELSE 0 END) AS BIGINT) AS aged_compaction_queue_count
FROM status_rollup_rows
";

pub(super) const STATUS_REASON_SUMMARY_SQL: &str = r"
WITH counts AS (
    SELECT
        CAST(COUNT(*) AS BIGINT) AS affected_corpus_count,
        CAST(SUM(CASE WHEN readable THEN 1 ELSE 0 END) AS BIGINT) AS readable_corpus_count
    FROM status_reason_rows
),
primary_reason AS (
    SELECT
        code,
        severity,
        action
    FROM status_reason_rows
    ORDER BY severity_priority ASC, code_priority ASC
    LIMIT 1
)
SELECT
    primary_reason.code,
    primary_reason.severity,
    primary_reason.action,
    counts.affected_corpus_count,
    counts.readable_corpus_count,
    CAST(counts.affected_corpus_count - counts.readable_corpus_count AS BIGINT) AS blocking_corpus_count
FROM primary_reason
CROSS JOIN counts
";

pub(super) const REPO_READ_PRESSURE_SUMMARY_SQL: &str = r"
SELECT
    CAST(budget AS BIGINT) AS budget,
    CAST(in_flight AS BIGINT) AS in_flight,
    captured_at,
    CAST(requested_repo_count AS BIGINT) AS requested_repo_count,
    CAST(searchable_repo_count AS BIGINT) AS searchable_repo_count,
    CAST(parallelism AS BIGINT) AS parallelism,
    fanout_capped
FROM repo_read_pressure_rows
LIMIT 1
";

pub(super) const QUERY_TELEMETRY_SUMMARY_SQL: &str = r"
SELECT
    CAST(COUNT(*) AS BIGINT) AS corpus_count,
    MAX(captured_at) AS latest_captured_at,
    CAST(SUM(CASE WHEN source = 'scan' THEN 1 ELSE 0 END) AS BIGINT) AS scan_count,
    CAST(SUM(CASE WHEN source = 'fts' THEN 1 ELSE 0 END) AS BIGINT) AS fts_count,
    CAST(SUM(CASE WHEN source = 'fts_fallback_scan' THEN 1 ELSE 0 END) AS BIGINT) AS fts_fallback_scan_count,
    CAST(SUM(rows_scanned) AS BIGINT) AS total_rows_scanned,
    CAST(SUM(matched_rows) AS BIGINT) AS total_matched_rows,
    CAST(SUM(result_count) AS BIGINT) AS total_result_count,
    CAST(MAX(batch_row_limit) AS BIGINT) AS max_batch_row_limit,
    CAST(MAX(recall_limit_rows) AS BIGINT) AS max_recall_limit_rows,
    CAST(MAX(working_set_budget_rows) AS BIGINT) AS max_working_set_budget_rows,
    CAST(MAX(trim_threshold_rows) AS BIGINT) AS max_trim_threshold_rows,
    CAST(MAX(peak_working_set_rows) AS BIGINT) AS max_peak_working_set_rows,
    CAST(SUM(trim_count) AS BIGINT) AS total_trim_count,
    CAST(SUM(dropped_candidate_count) AS BIGINT) AS total_dropped_candidate_count
FROM query_telemetry_rows
";

pub(super) const QUERY_TELEMETRY_SCOPE_SQL: &str = r"
SELECT
    scope,
    CAST(COUNT(*) AS BIGINT) AS corpus_count,
    MAX(captured_at) AS latest_captured_at,
    CAST(SUM(CASE WHEN source = 'scan' THEN 1 ELSE 0 END) AS BIGINT) AS scan_count,
    CAST(SUM(CASE WHEN source = 'fts' THEN 1 ELSE 0 END) AS BIGINT) AS fts_count,
    CAST(SUM(CASE WHEN source = 'fts_fallback_scan' THEN 1 ELSE 0 END) AS BIGINT) AS fts_fallback_scan_count,
    CAST(SUM(rows_scanned) AS BIGINT) AS total_rows_scanned,
    CAST(SUM(matched_rows) AS BIGINT) AS total_matched_rows,
    CAST(SUM(result_count) AS BIGINT) AS total_result_count,
    CAST(MAX(batch_row_limit) AS BIGINT) AS max_batch_row_limit,
    CAST(MAX(recall_limit_rows) AS BIGINT) AS max_recall_limit_rows,
    CAST(MAX(working_set_budget_rows) AS BIGINT) AS max_working_set_budget_rows,
    CAST(MAX(trim_threshold_rows) AS BIGINT) AS max_trim_threshold_rows,
    CAST(MAX(peak_working_set_rows) AS BIGINT) AS max_peak_working_set_rows,
    CAST(SUM(trim_count) AS BIGINT) AS total_trim_count,
    CAST(SUM(dropped_candidate_count) AS BIGINT) AS total_dropped_candidate_count
FROM query_telemetry_rows
WHERE scope IS NOT NULL AND scope <> ''
GROUP BY scope
ORDER BY scope ASC
";
