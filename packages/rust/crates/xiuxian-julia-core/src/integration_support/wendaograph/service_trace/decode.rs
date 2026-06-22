use std::io::Cursor;

use arrow::array::{Array, Float64Array, Int64Array, StringArray};
use arrow::ipc::reader::StreamReader;
use serde_json::{Value, json};

pub(super) fn candidate_input_discovery_json(payload: &str) -> Result<Value, String> {
    if payload.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(payload)
        .map_err(|error| format!("parse SearchStrategyFlow candidate discovery receipt: {error}"))
}

pub(super) fn decode_query_understanding_trace_rows(payload: &[u8]) -> Result<Vec<Value>, String> {
    let batches = decode_arrow_ipc_batches(payload, "query_understanding")?;
    let mut rows = Vec::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            rows.push(json!({
                "flowId": string_value(&batch, "flow_id", row_index)?,
                "intentId": string_value(&batch, "intent_id", row_index)?,
                "signalId": string_value(&batch, "signal_id", row_index)?,
                "signalKind": string_value(&batch, "signal_kind", row_index)?,
                "signalValue": string_value(&batch, "signal_value", row_index)?,
                "confidence": float_value(&batch, "confidence", row_index)?,
                "routeHint": string_value(&batch, "route_hint", row_index)?,
                "requiredEvidence": string_value(&batch, "required_evidence", row_index)?,
                "ambiguity": float_value(&batch, "ambiguity", row_index)?,
                "weight": float_value(&batch, "weight", row_index)?,
                "recommendedLoopBudget": int_value(&batch, "recommended_loop_budget", row_index)?,
                "recommendedJudgementBudget": int_value(&batch, "recommended_judgement_budget", row_index)?,
                "recommendedBeamWidth": int_value(&batch, "recommended_beam_width", row_index)?,
                "reason": string_value(&batch, "reason", row_index)?,
            }));
        }
    }
    Ok(rows)
}

pub(super) fn arrow_ipc_row_count(payload: &[u8]) -> Result<usize, String> {
    decode_arrow_ipc_batches(payload, "ontology_registry").map(|batches| {
        batches
            .iter()
            .map(arrow::record_batch::RecordBatch::num_rows)
            .sum()
    })
}

fn decode_arrow_ipc_batches(
    payload: &[u8],
    label: &str,
) -> Result<Vec<arrow::record_batch::RecordBatch>, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .map_err(|error| format!("decode SearchStrategyFlow {label} Arrow IPC: {error}"))?;
    reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode SearchStrategyFlow {label} Arrow IPC batch: {error}"))
}

fn string_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<String, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Utf8"))?;
    if array.is_null(row_index) {
        return Ok(String::new());
    }
    Ok(array.value(row_index).to_owned())
}

fn float_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<f64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Float64"))?;
    if array.is_null(row_index) {
        return Ok(0.0);
    }
    Ok(array.value(row_index))
}

fn int_value(
    batch: &arrow::record_batch::RecordBatch,
    column: &str,
    row_index: usize,
) -> Result<i64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("query_understanding missing `{column}`"))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| format!("query_understanding `{column}` must be Int64"))?;
    if array.is_null(row_index) {
        return Ok(0);
    }
    Ok(array.value(row_index))
}
