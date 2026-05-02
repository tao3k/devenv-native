use std::sync::Arc;

use arrow::array::{ArrayRef, BooleanArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

use crate::gateway::studio::types::search_index::diagnostics::labels::{
    bounded_u64_to_i64, query_telemetry_source_label, status_reason_action_label,
    status_reason_code_label, status_reason_severity_label,
};
use crate::gateway::studio::types::search_index::status::{
    response_reason_code_priority, response_reason_severity_priority,
};
use crate::search::SearchPlaneStatusSnapshot;
use xiuxian_db_store::EngineRecordBatch;

pub(super) fn status_snapshot_relation(
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

pub(super) fn query_telemetry_relation(
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

pub(super) fn status_reason_relation(
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

pub(super) fn repo_read_pressure_relation(
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

pub(super) struct StatusReasonRelationColumns {
    code: Vec<String>,
    severity: Vec<String>,
    action: Vec<String>,
    readable: Vec<bool>,
    severity_priority: Vec<i64>,
    code_priority: Vec<i64>,
}

pub(super) struct QueryTelemetryRelationColumns {
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

pub(super) fn collect_status_reason_relation_columns(
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
            .map(|reason| i64::from(response_reason_severity_priority(reason.severity.into())))
            .collect(),
        code_priority: reasons
            .iter()
            .map(|reason| i64::from(response_reason_code_priority(reason.code.into())))
            .collect(),
    })
}

pub(super) fn collect_query_telemetry_relation_columns(
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
