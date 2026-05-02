use std::sync::Arc;

use arrow::array::{Array, BooleanArray, Int64Array, StringArray};

use crate::gateway::studio::types::search_index::definitions as search_index;
use xiuxian_db_store::EngineRecordBatch;

use super::SearchIndexDiagnosticsRollup;

pub(super) fn decode_status_rollup(
    batches: &[EngineRecordBatch],
) -> Result<SearchIndexDiagnosticsRollup, String> {
    let total = decode_rollup_value(batches, "total")?;
    let idle = decode_rollup_value(batches, "idle")?;
    let indexing = decode_rollup_value(batches, "indexing")?;
    let ready = decode_rollup_value(batches, "ready")?;
    let degraded = decode_rollup_value(batches, "degraded")?;
    let failed = decode_rollup_value(batches, "failed")?;
    let compaction_pending = decode_rollup_value(batches, "compaction_pending")?;
    let prewarm_running_count = decode_rollup_value(batches, "prewarm_running_count")?;
    let prewarm_queued_corpus_count = decode_rollup_value(batches, "prewarm_queued_corpus_count")?;
    let max_prewarm_queue_depth = decode_rollup_value(batches, "max_prewarm_queue_depth")?;
    let compaction_running_count = decode_rollup_value(batches, "compaction_running_count")?;
    let compaction_queued_corpus_count =
        decode_rollup_value(batches, "compaction_queued_corpus_count")?;
    let max_compaction_queue_depth = decode_rollup_value(batches, "max_compaction_queue_depth")?;
    let compaction_pending_count = decode_rollup_value(batches, "compaction_pending_count")?;
    let aged_compaction_queue_count = decode_rollup_value(batches, "aged_compaction_queue_count")?;

    let maintenance_summary = if prewarm_running_count == 0
        && prewarm_queued_corpus_count == 0
        && compaction_running_count == 0
        && compaction_queued_corpus_count == 0
        && compaction_pending_count == 0
        && aged_compaction_queue_count == 0
    {
        None
    } else {
        Some(search_index::SearchIndexAggregateMaintenanceSummary {
            prewarm_running_count,
            prewarm_queued_corpus_count,
            max_prewarm_queue_depth: u32::try_from(max_prewarm_queue_depth).unwrap_or(u32::MAX),
            compaction_running_count,
            compaction_queued_corpus_count,
            max_compaction_queue_depth: u32::try_from(max_compaction_queue_depth)
                .unwrap_or(u32::MAX),
            compaction_pending_count,
            aged_compaction_queue_count,
        })
    };

    Ok(SearchIndexDiagnosticsRollup {
        total,
        idle,
        indexing,
        ready,
        degraded,
        failed,
        compaction_pending,
        maintenance_summary,
    })
}

pub(super) fn decode_query_telemetry_summary(
    summary_batches: &[EngineRecordBatch],
    scope_batches: &[EngineRecordBatch],
) -> Result<search_index::SearchIndexAggregateQueryTelemetry, String> {
    let batch = first_non_empty_batch(summary_batches)
        .ok_or_else(|| "status diagnostics query telemetry summary returned no rows".to_string())?;
    Ok(search_index::SearchIndexAggregateQueryTelemetry {
        corpus_count: decode_usize_value(batch, "corpus_count", 0)?,
        latest_captured_at: decode_string_value(batch, "latest_captured_at", 0)?,
        scan_count: decode_usize_value(batch, "scan_count", 0)?,
        fts_count: decode_usize_value(batch, "fts_count", 0)?,
        fts_fallback_scan_count: decode_usize_value(batch, "fts_fallback_scan_count", 0)?,
        total_rows_scanned: decode_u64_value(batch, "total_rows_scanned", 0)?,
        total_matched_rows: decode_u64_value(batch, "total_matched_rows", 0)?,
        total_result_count: decode_u64_value(batch, "total_result_count", 0)?,
        max_batch_row_limit: decode_optional_u64_value(batch, "max_batch_row_limit", 0)?,
        max_recall_limit_rows: decode_optional_u64_value(batch, "max_recall_limit_rows", 0)?,
        max_working_set_budget_rows: decode_u64_value(batch, "max_working_set_budget_rows", 0)?,
        max_trim_threshold_rows: decode_u64_value(batch, "max_trim_threshold_rows", 0)?,
        max_peak_working_set_rows: decode_u64_value(batch, "max_peak_working_set_rows", 0)?,
        total_trim_count: decode_u64_value(batch, "total_trim_count", 0)?,
        total_dropped_candidate_count: decode_u64_value(batch, "total_dropped_candidate_count", 0)?,
        scopes: decode_query_telemetry_scope_summaries(scope_batches)?,
    })
}

pub(super) fn decode_status_reason_summary(
    batches: &[EngineRecordBatch],
) -> Result<Option<search_index::SearchIndexAggregateStatusReason>, String> {
    let Some(batch) = first_non_empty_batch(batches) else {
        return Ok(None);
    };
    Ok(Some(search_index::SearchIndexAggregateStatusReason {
        code: decode_status_reason_code(&decode_string_value(batch, "code", 0)?)?,
        severity: decode_status_reason_severity(&decode_string_value(batch, "severity", 0)?)?,
        action: decode_status_reason_action(&decode_string_value(batch, "action", 0)?)?,
        affected_corpus_count: decode_usize_value(batch, "affected_corpus_count", 0)?,
        readable_corpus_count: decode_usize_value(batch, "readable_corpus_count", 0)?,
        blocking_corpus_count: decode_usize_value(batch, "blocking_corpus_count", 0)?,
    }))
}

pub(super) fn decode_repo_read_pressure_summary(
    batches: &[EngineRecordBatch],
) -> Result<Option<search_index::SearchIndexRepoReadPressure>, String> {
    let Some(batch) = first_non_empty_batch(batches) else {
        return Ok(None);
    };
    Ok(Some(search_index::SearchIndexRepoReadPressure {
        budget: decode_u32_value(batch, "budget", 0)?,
        in_flight: decode_u32_value(batch, "in_flight", 0)?,
        captured_at: decode_optional_string_value(batch, "captured_at", 0)?,
        requested_repo_count: decode_optional_u32_value(batch, "requested_repo_count", 0)?,
        searchable_repo_count: decode_optional_u32_value(batch, "searchable_repo_count", 0)?,
        parallelism: decode_optional_u32_value(batch, "parallelism", 0)?,
        fanout_capped: decode_bool_value(batch, "fanout_capped", 0)?,
    }))
}

pub(super) fn decode_query_telemetry_scope_summaries(
    batches: &[EngineRecordBatch],
) -> Result<Vec<search_index::SearchIndexQueryTelemetryScopeSummary>, String> {
    let mut scopes = Vec::new();
    for batch in batches.iter().filter(|batch| batch.num_rows() > 0) {
        for row in 0..batch.num_rows() {
            scopes.push(search_index::SearchIndexQueryTelemetryScopeSummary {
                scope: decode_string_value(batch, "scope", row)?,
                corpus_count: decode_usize_value(batch, "corpus_count", row)?,
                latest_captured_at: decode_string_value(batch, "latest_captured_at", row)?,
                scan_count: decode_usize_value(batch, "scan_count", row)?,
                fts_count: decode_usize_value(batch, "fts_count", row)?,
                fts_fallback_scan_count: decode_usize_value(batch, "fts_fallback_scan_count", row)?,
                total_rows_scanned: decode_u64_value(batch, "total_rows_scanned", row)?,
                total_matched_rows: decode_u64_value(batch, "total_matched_rows", row)?,
                total_result_count: decode_u64_value(batch, "total_result_count", row)?,
                max_batch_row_limit: decode_optional_u64_value(batch, "max_batch_row_limit", row)?,
                max_recall_limit_rows: decode_optional_u64_value(
                    batch,
                    "max_recall_limit_rows",
                    row,
                )?,
                max_working_set_budget_rows: decode_u64_value(
                    batch,
                    "max_working_set_budget_rows",
                    row,
                )?,
                max_trim_threshold_rows: decode_u64_value(batch, "max_trim_threshold_rows", row)?,
                max_peak_working_set_rows: decode_u64_value(
                    batch,
                    "max_peak_working_set_rows",
                    row,
                )?,
                total_trim_count: decode_u64_value(batch, "total_trim_count", row)?,
                total_dropped_candidate_count: decode_u64_value(
                    batch,
                    "total_dropped_candidate_count",
                    row,
                )?,
            });
        }
    }
    Ok(scopes)
}

pub(super) fn decode_rollup_value(
    batches: &[EngineRecordBatch],
    column: &str,
) -> Result<usize, String> {
    let batch = first_non_empty_batch(batches)
        .ok_or_else(|| format!("status diagnostics query returned no rows for `{column}`"))?;
    decode_usize_value(batch, column, 0)
}

fn first_non_empty_batch(batches: &[EngineRecordBatch]) -> Option<&EngineRecordBatch> {
    batches.iter().find(|batch| batch.num_rows() > 0)
}

fn column_values<'a>(
    batch: &'a EngineRecordBatch,
    column: &str,
) -> Result<&'a Arc<dyn Array>, String> {
    let column_index = batch.schema().index_of(column).map_err(|error| {
        format!("missing status diagnostics column `{column}` in rollup batch: {error}")
    })?;
    Ok(batch.column(column_index))
}

pub(super) fn decode_usize_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<usize, String> {
    let value = decode_i64_value(batch, column, row)?;
    usize::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed usize"))
}

pub(super) fn decode_u64_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<u64, String> {
    let value = decode_i64_value(batch, column, row)?;
    u64::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u64"))
}

pub(super) fn decode_u32_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<u32, String> {
    let value = decode_i64_value(batch, column, row)?;
    u32::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u32"))
}

pub(super) fn decode_optional_u64_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<Option<u64>, String> {
    let Some(value) = decode_optional_i64_value(batch, column, row)? else {
        return Ok(None);
    };
    u64::try_from(value)
        .map(Some)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u64"))
}

pub(super) fn decode_optional_u32_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<Option<u32>, String> {
    let Some(value) = decode_optional_i64_value(batch, column, row)? else {
        return Ok(None);
    };
    u32::try_from(value)
        .map(Some)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u32"))
}

pub(super) fn decode_i64_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<i64, String> {
    let values = column_values(batch, column)?;
    let Some(values) = values.as_any().downcast_ref::<Int64Array>() else {
        return Err(format!(
            "unsupported status diagnostics column type for `{column}`: {:?}",
            values.data_type()
        ));
    };
    if values.is_null(row) {
        return Err(format!(
            "unexpected null status diagnostics value for `{column}`"
        ));
    }
    Ok(values.value(row))
}

pub(super) fn decode_optional_i64_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<Option<i64>, String> {
    let values = column_values(batch, column)?;
    let Some(values) = values.as_any().downcast_ref::<Int64Array>() else {
        return Err(format!(
            "unsupported status diagnostics column type for `{column}`: {:?}",
            values.data_type()
        ));
    };
    if values.is_null(row) {
        return Ok(None);
    }
    Ok(Some(values.value(row)))
}

pub(super) fn decode_string_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<String, String> {
    let values = column_values(batch, column)?;
    let Some(values) = values.as_any().downcast_ref::<StringArray>() else {
        return Err(format!(
            "unsupported status diagnostics column type for `{column}`: {:?}",
            values.data_type()
        ));
    };
    if values.is_null(row) {
        return Err(format!(
            "unexpected null status diagnostics value for `{column}`"
        ));
    }
    Ok(values.value(row).to_string())
}

pub(super) fn decode_optional_string_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<Option<String>, String> {
    let values = column_values(batch, column)?;
    let Some(values) = values.as_any().downcast_ref::<StringArray>() else {
        return Err(format!(
            "unsupported status diagnostics column type for `{column}`: {:?}",
            values.data_type()
        ));
    };
    if values.is_null(row) {
        return Ok(None);
    }
    Ok(Some(values.value(row).to_string()))
}

pub(super) fn decode_bool_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<bool, String> {
    let values = column_values(batch, column)?;
    let Some(values) = values.as_any().downcast_ref::<BooleanArray>() else {
        return Err(format!(
            "unsupported status diagnostics column type for `{column}`: {:?}",
            values.data_type()
        ));
    };
    if values.is_null(row) {
        return Err(format!(
            "unexpected null status diagnostics value for `{column}`"
        ));
    }
    Ok(values.value(row))
}

pub(super) fn decode_status_reason_code(
    value: &str,
) -> Result<search_index::SearchIndexStatusReasonCode, String> {
    match value {
        "warming_up" => Ok(search_index::SearchIndexStatusReasonCode::WarmingUp),
        "prewarming" => Ok(search_index::SearchIndexStatusReasonCode::Prewarming),
        "refreshing" => Ok(search_index::SearchIndexStatusReasonCode::Refreshing),
        "compacting" => Ok(search_index::SearchIndexStatusReasonCode::Compacting),
        "compaction_pending" => Ok(search_index::SearchIndexStatusReasonCode::CompactionPending),
        "build_failed" => Ok(search_index::SearchIndexStatusReasonCode::BuildFailed),
        "published_manifest_missing" => {
            Ok(search_index::SearchIndexStatusReasonCode::PublishedManifestMissing)
        }
        "published_revision_missing" => {
            Ok(search_index::SearchIndexStatusReasonCode::PublishedRevisionMissing)
        }
        "published_revision_mismatch" => {
            Ok(search_index::SearchIndexStatusReasonCode::PublishedRevisionMismatch)
        }
        "repo_index_failed" => Ok(search_index::SearchIndexStatusReasonCode::RepoIndexFailed),
        _ => Err(format!(
            "unsupported status diagnostics reason code label `{value}`"
        )),
    }
}

pub(super) fn decode_status_reason_severity(
    value: &str,
) -> Result<search_index::SearchIndexStatusSeverity, String> {
    match value {
        "info" => Ok(search_index::SearchIndexStatusSeverity::Info),
        "warning" => Ok(search_index::SearchIndexStatusSeverity::Warning),
        "error" => Ok(search_index::SearchIndexStatusSeverity::Error),
        _ => Err(format!(
            "unsupported status diagnostics severity label `{value}`"
        )),
    }
}

pub(super) fn decode_status_reason_action(
    value: &str,
) -> Result<search_index::SearchIndexStatusAction, String> {
    match value {
        "wait" => Ok(search_index::SearchIndexStatusAction::Wait),
        "retry_build" => Ok(search_index::SearchIndexStatusAction::RetryBuild),
        "resync_repo" => Ok(search_index::SearchIndexStatusAction::ResyncRepo),
        "inspect_repo_sync" => Ok(search_index::SearchIndexStatusAction::InspectRepoSync),
        _ => Err(format!(
            "unsupported status diagnostics action label `{value}`"
        )),
    }
}
