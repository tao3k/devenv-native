use std::sync::Arc;

use arrow::array::{Array, ArrayRef, BooleanArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

use super::definitions as search_index;
#[cfg(all(test, feature = "duckdb"))]
use crate::duckdb::LocalRelationEngineKind;
use crate::duckdb::{
    DataFusionLocalRelationEngine, LocalRelationEngine, LocalRelationRegistrationHint,
};
#[cfg(feature = "duckdb")]
use crate::duckdb::{DuckDbLocalRelationEngine, resolve_search_duckdb_runtime};
use crate::search::{SearchPlaneStatusSnapshot, SearchQueryTelemetrySource};
use xiuxian_db_store::EngineRecordBatch;

const STATUS_DIAGNOSTICS_TABLE: &str = "status_rollup_rows";
const STATUS_REASON_DIAGNOSTICS_TABLE: &str = "status_reason_rows";
const QUERY_TELEMETRY_DIAGNOSTICS_TABLE: &str = "query_telemetry_rows";
const REPO_READ_PRESSURE_DIAGNOSTICS_TABLE: &str = "repo_read_pressure_rows";

pub(crate) struct SearchIndexDiagnosticsSummary {
    pub(crate) rollup: SearchIndexDiagnosticsRollup,
    pub(crate) status_reason: Option<search_index::SearchIndexAggregateStatusReason>,
    pub(crate) query_telemetry_summary: Option<search_index::SearchIndexAggregateQueryTelemetry>,
    pub(crate) repo_read_pressure: Option<search_index::SearchIndexRepoReadPressure>,
}

pub(crate) struct SearchIndexDiagnosticsRollup {
    pub(crate) total: usize,
    pub(crate) idle: usize,
    pub(crate) indexing: usize,
    pub(crate) ready: usize,
    pub(crate) degraded: usize,
    pub(crate) failed: usize,
    pub(crate) compaction_pending: usize,
    pub(crate) maintenance_summary: Option<search_index::SearchIndexAggregateMaintenanceSummary>,
}

#[cfg(all(test, feature = "duckdb"))]
pub(crate) fn configured_status_diagnostics_engine_kind() -> Result<LocalRelationEngineKind, String>
{
    Ok(configured_status_diagnostics_engine()?.kind())
}

pub(crate) async fn summarize_status_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<SearchIndexDiagnosticsSummary, String> {
    #[cfg(feature = "duckdb")]
    let engine = configured_status_diagnostics_engine()?;
    #[cfg(not(feature = "duckdb"))]
    let engine = configured_status_diagnostics_engine();
    let (schema, batches) = status_snapshot_relation(snapshot)?;
    engine.register_record_batches(STATUS_DIAGNOSTICS_TABLE, schema, batches)?;
    let rollup_batches = engine
        .query_batches(STATUS_DIAGNOSTICS_SQL)
        .await
        .map_err(|error| format!("status diagnostics rollup query failed: {error}"))?;
    let rollup = decode_status_rollup(rollup_batches.as_slice())?;
    let status_reason = summarize_status_reason_diagnostics(snapshot, engine.as_ref()).await?;
    let query_telemetry_summary =
        summarize_query_telemetry_diagnostics(snapshot, engine.as_ref()).await?;
    let repo_read_pressure =
        summarize_repo_read_pressure_diagnostics(snapshot, engine.as_ref()).await?;
    Ok(SearchIndexDiagnosticsSummary {
        rollup,
        status_reason,
        query_telemetry_summary,
        repo_read_pressure,
    })
}

#[cfg(feature = "duckdb")]
fn configured_status_diagnostics_engine() -> Result<Box<dyn LocalRelationEngine>, String> {
    #[cfg(feature = "duckdb")]
    {
        let runtime = resolve_search_duckdb_runtime();
        if runtime.enabled {
            return DuckDbLocalRelationEngine::from_runtime(runtime)
                .map(|engine| Box::new(engine) as Box<dyn LocalRelationEngine>);
        }
    }

    Ok(Box::new(
        DataFusionLocalRelationEngine::new_with_information_schema(),
    ))
}

#[cfg(not(feature = "duckdb"))]
fn configured_status_diagnostics_engine() -> Box<dyn LocalRelationEngine> {
    Box::new(DataFusionLocalRelationEngine::new_with_information_schema())
}

async fn summarize_query_telemetry_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexAggregateQueryTelemetry>, String> {
    let Some((schema, batches)) = query_telemetry_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches_with_hint(
        QUERY_TELEMETRY_DIAGNOSTICS_TABLE,
        schema,
        batches,
        LocalRelationRegistrationHint::RepeatedUse,
    )?;
    let summary_batches = engine
        .query_batches(QUERY_TELEMETRY_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics query telemetry summary failed: {error}"))?;
    let scopes_batches = engine
        .query_batches(QUERY_TELEMETRY_SCOPE_SQL)
        .await
        .map_err(|error| {
            format!("status diagnostics query telemetry scope query failed: {error}")
        })?;
    Ok(Some(decode_query_telemetry_summary(
        summary_batches.as_slice(),
        scopes_batches.as_slice(),
    )?))
}

async fn summarize_status_reason_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexAggregateStatusReason>, String> {
    let Some((schema, batches)) = status_reason_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches(STATUS_REASON_DIAGNOSTICS_TABLE, schema, batches)?;
    let summary_batches = engine
        .query_batches(STATUS_REASON_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics status reason query failed: {error}"))?;
    decode_status_reason_summary(summary_batches.as_slice())
}

async fn summarize_repo_read_pressure_diagnostics(
    snapshot: &SearchPlaneStatusSnapshot,
    engine: &dyn LocalRelationEngine,
) -> Result<Option<search_index::SearchIndexRepoReadPressure>, String> {
    let Some((schema, batches)) = repo_read_pressure_relation(snapshot)? else {
        return Ok(None);
    };
    engine.register_record_batches(REPO_READ_PRESSURE_DIAGNOSTICS_TABLE, schema, batches)?;
    let summary_batches = engine
        .query_batches(REPO_READ_PRESSURE_SUMMARY_SQL)
        .await
        .map_err(|error| format!("status diagnostics repo read pressure query failed: {error}"))?;
    decode_repo_read_pressure_summary(summary_batches.as_slice())
}

fn status_snapshot_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<(SchemaRef, Vec<EngineRecordBatch>), String> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("corpus", DataType::Utf8, false),
        Field::new("phase", DataType::Utf8, false),
        Field::new("prewarm_running", DataType::Boolean, false),
        Field::new("prewarm_queue_depth", DataType::Int64, false),
        Field::new("compaction_running", DataType::Boolean, false),
        Field::new("compaction_queue_depth", DataType::Int64, false),
        Field::new("compaction_queue_aged", DataType::Boolean, false),
        Field::new("compaction_pending", DataType::Boolean, false),
    ]));
    let corpus = snapshot
        .corpora
        .iter()
        .map(|status| status.corpus.as_str())
        .collect::<Vec<_>>();
    let phase = snapshot
        .corpora
        .iter()
        .map(|status| match status.phase {
            crate::search::SearchPlanePhase::Idle => "idle",
            crate::search::SearchPlanePhase::Indexing => "indexing",
            crate::search::SearchPlanePhase::Ready => "ready",
            crate::search::SearchPlanePhase::Degraded => "degraded",
            crate::search::SearchPlanePhase::Failed => "failed",
        })
        .collect::<Vec<_>>();
    let prewarm_running = snapshot
        .corpora
        .iter()
        .map(|status| status.maintenance.prewarm_running)
        .collect::<Vec<_>>();
    let prewarm_queue_depth = snapshot
        .corpora
        .iter()
        .map(|status| i64::from(status.maintenance.prewarm_queue_depth))
        .collect::<Vec<_>>();
    let compaction_running = snapshot
        .corpora
        .iter()
        .map(|status| status.maintenance.compaction_running)
        .collect::<Vec<_>>();
    let compaction_queue_depth = snapshot
        .corpora
        .iter()
        .map(|status| i64::from(status.maintenance.compaction_queue_depth))
        .collect::<Vec<_>>();
    let compaction_queue_aged = snapshot
        .corpora
        .iter()
        .map(|status| status.maintenance.compaction_queue_aged.is_aged())
        .collect::<Vec<_>>();
    let compaction_pending = snapshot
        .corpora
        .iter()
        .map(|status| status.maintenance.compaction_pending)
        .collect::<Vec<_>>();

    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(corpus)) as ArrayRef,
            Arc::new(StringArray::from(phase)) as ArrayRef,
            Arc::new(BooleanArray::from(prewarm_running)) as ArrayRef,
            Arc::new(Int64Array::from(prewarm_queue_depth)) as ArrayRef,
            Arc::new(BooleanArray::from(compaction_running)) as ArrayRef,
            Arc::new(Int64Array::from(compaction_queue_depth)) as ArrayRef,
            Arc::new(BooleanArray::from(compaction_queue_aged)) as ArrayRef,
            Arc::new(BooleanArray::from(compaction_pending)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("failed to build status diagnostics relation batch: {error}"))?;

    Ok((schema, vec![batch]))
}

fn query_telemetry_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(columns) = collect_query_telemetry_relation_columns(snapshot) else {
        return Ok(None);
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("captured_at", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, true),
        Field::new("source", DataType::Utf8, false),
        Field::new("batch_count", DataType::Int64, false),
        Field::new("rows_scanned", DataType::Int64, false),
        Field::new("matched_rows", DataType::Int64, false),
        Field::new("result_count", DataType::Int64, false),
        Field::new("batch_row_limit", DataType::Int64, true),
        Field::new("recall_limit_rows", DataType::Int64, true),
        Field::new("working_set_budget_rows", DataType::Int64, false),
        Field::new("trim_threshold_rows", DataType::Int64, false),
        Field::new("peak_working_set_rows", DataType::Int64, false),
        Field::new("trim_count", DataType::Int64, false),
        Field::new("dropped_candidate_count", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(columns.captured_at)) as ArrayRef,
            Arc::new(StringArray::from(columns.scope)) as ArrayRef,
            Arc::new(StringArray::from(columns.source)) as ArrayRef,
            Arc::new(Int64Array::from(columns.batch_count)) as ArrayRef,
            Arc::new(Int64Array::from(columns.rows_scanned)) as ArrayRef,
            Arc::new(Int64Array::from(columns.matched_rows)) as ArrayRef,
            Arc::new(Int64Array::from(columns.result_count)) as ArrayRef,
            Arc::new(Int64Array::from(columns.batch_row_limit)) as ArrayRef,
            Arc::new(Int64Array::from(columns.recall_limit_rows)) as ArrayRef,
            Arc::new(Int64Array::from(columns.working_set_budget_rows)) as ArrayRef,
            Arc::new(Int64Array::from(columns.trim_threshold_rows)) as ArrayRef,
            Arc::new(Int64Array::from(columns.peak_working_set_rows)) as ArrayRef,
            Arc::new(Int64Array::from(columns.trim_count)) as ArrayRef,
            Arc::new(Int64Array::from(columns.dropped_candidate_count)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("failed to build query telemetry diagnostics batch: {error}"))?;

    Ok(Some((schema, vec![batch])))
}

fn status_reason_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(columns) = collect_status_reason_relation_columns(snapshot) else {
        return Ok(None);
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("code", DataType::Utf8, false),
        Field::new("severity", DataType::Utf8, false),
        Field::new("action", DataType::Utf8, false),
        Field::new("readable", DataType::Boolean, false),
        Field::new("severity_priority", DataType::Int64, false),
        Field::new("code_priority", DataType::Int64, false),
    ]));

    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(columns.code)) as ArrayRef,
            Arc::new(StringArray::from(columns.severity)) as ArrayRef,
            Arc::new(StringArray::from(columns.action)) as ArrayRef,
            Arc::new(BooleanArray::from(columns.readable)) as ArrayRef,
            Arc::new(Int64Array::from(columns.severity_priority)) as ArrayRef,
            Arc::new(Int64Array::from(columns.code_priority)) as ArrayRef,
        ],
    )
    .map_err(|error| format!("failed to build status reason diagnostics batch: {error}"))?;

    Ok(Some((schema, vec![batch])))
}

fn repo_read_pressure_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(pressure) = snapshot.repo_read_pressure.as_ref() else {
        return Ok(None);
    };

    let schema = Arc::new(Schema::new(vec![
        Field::new("budget", DataType::Int64, false),
        Field::new("in_flight", DataType::Int64, false),
        Field::new("captured_at", DataType::Utf8, true),
        Field::new("requested_repo_count", DataType::Int64, true),
        Field::new("searchable_repo_count", DataType::Int64, true),
        Field::new("parallelism", DataType::Int64, true),
        Field::new("fanout_capped", DataType::Boolean, false),
    ]));

    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(Int64Array::from(vec![i64::from(pressure.budget)])) as ArrayRef,
            Arc::new(Int64Array::from(vec![i64::from(pressure.in_flight)])) as ArrayRef,
            Arc::new(StringArray::from(vec![pressure.captured_at.clone()])) as ArrayRef,
            Arc::new(Int64Array::from(vec![
                pressure.requested_repo_count.map(i64::from),
            ])) as ArrayRef,
            Arc::new(Int64Array::from(vec![
                pressure.searchable_repo_count.map(i64::from),
            ])) as ArrayRef,
            Arc::new(Int64Array::from(vec![pressure.parallelism.map(i64::from)])) as ArrayRef,
            Arc::new(BooleanArray::from(vec![pressure.fanout_capped])) as ArrayRef,
        ],
    )
    .map_err(|error| format!("failed to build repo read pressure diagnostics batch: {error}"))?;

    Ok(Some((schema, vec![batch])))
}

struct StatusReasonRelationColumns {
    code: Vec<String>,
    severity: Vec<String>,
    action: Vec<String>,
    readable: Vec<bool>,
    severity_priority: Vec<i64>,
    code_priority: Vec<i64>,
}

struct QueryTelemetryRelationColumns {
    captured_at: Vec<String>,
    scope: Vec<Option<String>>,
    source: Vec<String>,
    batch_count: Vec<i64>,
    rows_scanned: Vec<i64>,
    matched_rows: Vec<i64>,
    result_count: Vec<i64>,
    batch_row_limit: Vec<Option<i64>>,
    recall_limit_rows: Vec<Option<i64>>,
    working_set_budget_rows: Vec<i64>,
    trim_threshold_rows: Vec<i64>,
    peak_working_set_rows: Vec<i64>,
    trim_count: Vec<i64>,
    dropped_candidate_count: Vec<i64>,
}

fn collect_status_reason_relation_columns(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Option<StatusReasonRelationColumns> {
    let reasons = snapshot
        .corpora
        .iter()
        .filter_map(|status| status.status_reason.as_ref())
        .collect::<Vec<_>>();
    if reasons.is_empty() {
        return None;
    }

    Some(StatusReasonRelationColumns {
        code: reasons
            .iter()
            .map(|reason| status_reason_code_label(reason.code.into()).to_string())
            .collect(),
        severity: reasons
            .iter()
            .map(|reason| status_reason_severity_label(reason.severity.into()).to_string())
            .collect(),
        action: reasons
            .iter()
            .map(|reason| status_reason_action_label(reason.action.into()).to_string())
            .collect(),
        readable: reasons.iter().map(|reason| reason.readable).collect(),
        severity_priority: reasons
            .iter()
            .map(|reason| {
                i64::from(super::status::response_reason_severity_priority(
                    reason.severity.into(),
                ))
            })
            .collect(),
        code_priority: reasons
            .iter()
            .map(|reason| {
                i64::from(super::status::response_reason_code_priority(
                    reason.code.into(),
                ))
            })
            .collect(),
    })
}

fn collect_query_telemetry_relation_columns(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Option<QueryTelemetryRelationColumns> {
    let telemetry = snapshot
        .corpora
        .iter()
        .filter_map(|status| status.last_query_telemetry.as_ref())
        .collect::<Vec<_>>();
    if telemetry.is_empty() {
        return None;
    }

    Some(QueryTelemetryRelationColumns {
        captured_at: telemetry
            .iter()
            .map(|entry| entry.captured_at.clone())
            .collect(),
        scope: telemetry.iter().map(|entry| entry.scope.clone()).collect(),
        source: telemetry
            .iter()
            .map(|entry| query_telemetry_source_label(entry.source).to_string())
            .collect(),
        batch_count: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.batch_count))
            .collect(),
        rows_scanned: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.rows_scanned))
            .collect(),
        matched_rows: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.matched_rows))
            .collect(),
        result_count: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.result_count))
            .collect(),
        batch_row_limit: telemetry
            .iter()
            .map(|entry| entry.batch_row_limit.map(bounded_u64_to_i64))
            .collect(),
        recall_limit_rows: telemetry
            .iter()
            .map(|entry| entry.recall_limit_rows.map(bounded_u64_to_i64))
            .collect(),
        working_set_budget_rows: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.working_set_budget_rows))
            .collect(),
        trim_threshold_rows: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.trim_threshold_rows))
            .collect(),
        peak_working_set_rows: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.peak_working_set_rows))
            .collect(),
        trim_count: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.trim_count))
            .collect(),
        dropped_candidate_count: telemetry
            .iter()
            .map(|entry| bounded_u64_to_i64(entry.dropped_candidate_count))
            .collect(),
    })
}

fn query_telemetry_source_label(source: SearchQueryTelemetrySource) -> &'static str {
    match source {
        SearchQueryTelemetrySource::Scan => "scan",
        SearchQueryTelemetrySource::Fts => "fts",
        SearchQueryTelemetrySource::FtsFallbackScan => "fts_fallback_scan",
    }
}

fn status_reason_code_label(code: search_index::SearchIndexStatusReasonCode) -> &'static str {
    match code {
        search_index::SearchIndexStatusReasonCode::WarmingUp => "warming_up",
        search_index::SearchIndexStatusReasonCode::Prewarming => "prewarming",
        search_index::SearchIndexStatusReasonCode::Refreshing => "refreshing",
        search_index::SearchIndexStatusReasonCode::Compacting => "compacting",
        search_index::SearchIndexStatusReasonCode::CompactionPending => "compaction_pending",
        search_index::SearchIndexStatusReasonCode::BuildFailed => "build_failed",
        search_index::SearchIndexStatusReasonCode::PublishedManifestMissing => {
            "published_manifest_missing"
        }
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMissing => {
            "published_revision_missing"
        }
        search_index::SearchIndexStatusReasonCode::PublishedRevisionMismatch => {
            "published_revision_mismatch"
        }
        search_index::SearchIndexStatusReasonCode::RepoIndexFailed => "repo_index_failed",
    }
}

fn status_reason_severity_label(severity: search_index::SearchIndexStatusSeverity) -> &'static str {
    match severity {
        search_index::SearchIndexStatusSeverity::Info => "info",
        search_index::SearchIndexStatusSeverity::Warning => "warning",
        search_index::SearchIndexStatusSeverity::Error => "error",
    }
}

fn status_reason_action_label(action: search_index::SearchIndexStatusAction) -> &'static str {
    match action {
        search_index::SearchIndexStatusAction::Wait => "wait",
        search_index::SearchIndexStatusAction::RetryBuild => "retry_build",
        search_index::SearchIndexStatusAction::ResyncRepo => "resync_repo",
        search_index::SearchIndexStatusAction::InspectRepoSync => "inspect_repo_sync",
    }
}

fn bounded_u64_to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn decode_status_rollup(
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

fn decode_query_telemetry_summary(
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

fn decode_status_reason_summary(
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

fn decode_repo_read_pressure_summary(
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

fn decode_query_telemetry_scope_summaries(
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

fn decode_rollup_value(batches: &[EngineRecordBatch], column: &str) -> Result<usize, String> {
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

fn decode_usize_value(
    batch: &EngineRecordBatch,
    column: &str,
    row: usize,
) -> Result<usize, String> {
    let value = decode_i64_value(batch, column, row)?;
    usize::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed usize"))
}

fn decode_u64_value(batch: &EngineRecordBatch, column: &str, row: usize) -> Result<u64, String> {
    let value = decode_i64_value(batch, column, row)?;
    u64::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u64"))
}

fn decode_u32_value(batch: &EngineRecordBatch, column: &str, row: usize) -> Result<u32, String> {
    let value = decode_i64_value(batch, column, row)?;
    u32::try_from(value)
        .map_err(|_| format!("status diagnostics value for `{column}` overflowed u32"))
}

fn decode_optional_u64_value(
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

fn decode_optional_u32_value(
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

fn decode_i64_value(batch: &EngineRecordBatch, column: &str, row: usize) -> Result<i64, String> {
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

fn decode_optional_i64_value(
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

fn decode_string_value(
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

fn decode_optional_string_value(
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

fn decode_bool_value(batch: &EngineRecordBatch, column: &str, row: usize) -> Result<bool, String> {
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

fn decode_status_reason_code(
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

fn decode_status_reason_severity(
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

fn decode_status_reason_action(
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

const STATUS_DIAGNOSTICS_SQL: &str = r"
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

const STATUS_REASON_SUMMARY_SQL: &str = r"
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

const REPO_READ_PRESSURE_SUMMARY_SQL: &str = r"
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

const QUERY_TELEMETRY_SUMMARY_SQL: &str = r"
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

const QUERY_TELEMETRY_SCOPE_SQL: &str = r"
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
