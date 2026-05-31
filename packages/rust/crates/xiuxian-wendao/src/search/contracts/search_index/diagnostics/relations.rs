//! Arrow read-model relations for search-index diagnostics.

use std::{collections::HashMap, sync::Arc};

use arrow::array::{ArrayRef, BooleanArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::SchemaRef;

use crate::search::SearchPlaneStatusSnapshot;
use crate::search::contracts::search_index::diagnostics::labels::{
    bounded_u64_to_i64, query_telemetry_source_label, status_reason_action_label,
    status_reason_code_label, status_reason_severity_label,
};
use crate::search::contracts::search_index::status::{
    response_reason_code_priority, response_reason_severity_priority,
};
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, EngineRecordBatch, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

/// Arrow table name for search status rollup diagnostics.
pub const STATUS_DIAGNOSTICS_TABLE: &str = "status_rollup_rows";
/// Arrow table name for status reason diagnostics.
pub const STATUS_REASON_DIAGNOSTICS_TABLE: &str = "status_reason_rows";
/// Arrow table name for query telemetry diagnostics.
pub const QUERY_TELEMETRY_DIAGNOSTICS_TABLE: &str = "query_telemetry_rows";
/// Arrow table name for repo read-pressure diagnostics.
pub const REPO_READ_PRESSURE_DIAGNOSTICS_TABLE: &str = "repo_read_pressure_rows";

pub(super) fn status_snapshot_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<(SchemaRef, Vec<EngineRecordBatch>), String> {
    let contract = status_snapshot_contract();
    let schema = diagnostics_schema_ref(&contract);
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
    validate_diagnostics_batch(&batch, &contract, "status diagnostics relation schema")?;

    Ok((schema, vec![batch]))
}

pub(super) fn query_telemetry_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(columns) = collect_query_telemetry_relation_columns(snapshot) else {
        return Ok(None);
    };

    let contract = query_telemetry_contract();
    let schema = diagnostics_schema_ref(&contract);

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
    validate_diagnostics_batch(
        &batch,
        &contract,
        "query telemetry diagnostics relation schema",
    )?;

    Ok(Some((schema, vec![batch])))
}

pub(super) fn status_reason_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(columns) = collect_status_reason_relation_columns(snapshot) else {
        return Ok(None);
    };

    let contract = status_reason_contract();
    let schema = diagnostics_schema_ref(&contract);

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
    validate_diagnostics_batch(
        &batch,
        &contract,
        "status reason diagnostics relation schema",
    )?;

    Ok(Some((schema, vec![batch])))
}

pub(super) fn repo_read_pressure_relation(
    snapshot: &SearchPlaneStatusSnapshot,
) -> Result<Option<(SchemaRef, Vec<EngineRecordBatch>)>, String> {
    let Some(pressure) = snapshot.repo_read_pressure.as_ref() else {
        return Ok(None);
    };

    let contract = repo_read_pressure_contract();
    let schema = diagnostics_schema_ref(&contract);

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
    validate_diagnostics_batch(
        &batch,
        &contract,
        "repo read pressure diagnostics relation schema",
    )?;

    Ok(Some((schema, vec![batch])))
}

/// Build the Arrow schema contract for status snapshot diagnostics.
pub fn status_snapshot_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STATUS_DIAGNOSTICS_TABLE,
        true,
        vec![
            utf8_column("corpus"),
            utf8_column("phase"),
            boolean_column("prewarm_running"),
            int64_column("prewarm_queue_depth"),
            boolean_column("compaction_running"),
            int64_column("compaction_queue_depth"),
            boolean_column("compaction_queue_aged"),
            boolean_column("compaction_pending"),
        ],
    )
}

/// Build the Arrow schema contract for query telemetry diagnostics.
pub fn query_telemetry_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        QUERY_TELEMETRY_DIAGNOSTICS_TABLE,
        true,
        vec![
            utf8_column("captured_at"),
            nullable_utf8_column("scope"),
            utf8_column("source"),
            int64_column("batch_count"),
            int64_column("rows_scanned"),
            int64_column("matched_rows"),
            int64_column("result_count"),
            nullable_int64_column("batch_row_limit"),
            nullable_int64_column("recall_limit_rows"),
            int64_column("working_set_budget_rows"),
            int64_column("trim_threshold_rows"),
            int64_column("peak_working_set_rows"),
            int64_column("trim_count"),
            int64_column("dropped_candidate_count"),
        ],
    )
}

/// Build the Arrow schema contract for status reason diagnostics.
pub fn status_reason_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        STATUS_REASON_DIAGNOSTICS_TABLE,
        true,
        vec![
            utf8_column("code"),
            utf8_column("severity"),
            utf8_column("action"),
            boolean_column("readable"),
            int64_column("severity_priority"),
            int64_column("code_priority"),
        ],
    )
}

/// Build the Arrow schema contract for repo read-pressure diagnostics.
pub fn repo_read_pressure_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        REPO_READ_PRESSURE_DIAGNOSTICS_TABLE,
        true,
        vec![
            int64_column("budget"),
            int64_column("in_flight"),
            nullable_utf8_column("captured_at"),
            nullable_int64_column("requested_repo_count"),
            nullable_int64_column("searchable_repo_count"),
            nullable_int64_column("parallelism"),
            boolean_column("fanout_capped"),
        ],
    )
}

/// Build a diagnostics Arrow schema reference with Wendao table metadata.
pub fn diagnostics_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

fn validate_diagnostics_batch(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("{context}: {error}"))
}

fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

fn int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int64)
}

fn nullable_int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int64)
}

fn boolean_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
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
