//! Response-bundle decoders for the `SearchStrategyFlow` Flight service.

use std::io::Cursor;

use arrow::array::{
    Array, BinaryArray, BooleanArray, Float64Array, Int64Array, LargeBinaryArray, LargeListArray,
    ListArray, StringArray, UInt8Array,
};
use arrow::ipc::reader::StreamReader;
use arrow::record_batch::RecordBatch;

use super::types::{
    SearchStrategyFlowFrontierRow, SearchStrategyFlowServiceCandidateRow,
    SearchStrategyFlowServicePlannerActionRow, SearchStrategyFlowServiceResponse,
};
use crate::integration_support::search_strategy_flow_flight::constants::{
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
    WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
};

/// Decode a full `SearchStrategyFlow` service response.
///
/// # Errors
///
/// Returns an error when the response bundle misses required payload columns or
/// any contained Arrow IPC payload cannot be decoded.
pub fn decode_search_strategy_flow_service_response(
    batches: &[RecordBatch],
) -> Result<SearchStrategyFlowServiceResponse, String> {
    let frontier = decode_search_strategy_flow_frontier_rows(batches)?;
    let Some(candidate_batches) = response_bundle_payload_batches(
        batches,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
    )?
    else {
        return Ok(SearchStrategyFlowServiceResponse {
            candidates: Vec::new(),
            transition_count: 0,
            frontier,
            planner_actions: Vec::new(),
        });
    };
    let transition_batches = response_bundle_payload_batches(
        batches,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
    )?
    .ok_or_else(|| "SearchStrategyFlow response bundle missing transition payloads".to_string())?;
    let planner_action_batches = response_bundle_payload_batches(
        batches,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
    )?
    .ok_or_else(|| {
        "SearchStrategyFlow response bundle missing planner action payloads".to_string()
    })?;
    Ok(SearchStrategyFlowServiceResponse {
        candidates: decode_candidate_batches(&candidate_batches)?,
        transition_count: transition_batches
            .iter()
            .map(RecordBatch::num_rows)
            .sum::<usize>(),
        frontier,
        planner_actions: decode_planner_action_batches(&planner_action_batches)?,
    })
}

/// Decode `strategy_frontier` response batches returned by the service.
///
/// # Errors
///
/// Returns an error when the response is empty, misses required columns, or a
/// required value cannot be decoded from its Arrow type.
pub fn decode_search_strategy_flow_frontier_rows(
    batches: &[RecordBatch],
) -> Result<Vec<SearchStrategyFlowFrontierRow>, String> {
    let frontier_batches = search_strategy_flow_frontier_response_batches(batches)?;
    let rows = frontier_batches
        .iter()
        .map(decode_frontier_batch)
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err("SearchStrategyFlow service response returned zero frontier rows".to_string());
    }
    Ok(rows)
}

fn search_strategy_flow_frontier_response_batches(
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, String> {
    response_bundle_payload_batches(
        batches,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
    )
    .map(|bundled_batches| bundled_batches.unwrap_or_else(|| batches.to_vec()))
}

fn response_bundle_payload_batches(
    batches: &[RecordBatch],
    column_name: &str,
) -> Result<Option<Vec<RecordBatch>>, String> {
    let mut bundled_batches = Vec::new();
    let mut saw_direct_batch = false;
    for batch in batches {
        if has_response_bundle_payload_column(batch) {
            require_response_bundle_columns(batch)?;
            bundled_batches.extend(decode_response_bundle_payload_batches(batch, column_name)?);
        } else {
            saw_direct_batch = true;
        }
    }
    if !bundled_batches.is_empty() {
        if saw_direct_batch {
            return Err(
                "SearchStrategyFlow service response mixed bundled and direct frontier batches"
                    .to_string(),
            );
        }
        return Ok(Some(bundled_batches));
    }
    Ok(None)
}

fn decode_response_bundle_payload_batches(
    batch: &RecordBatch,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    require_response_bundle_columns(batch)?;
    let column = batch
        .column_by_name(column_name)
        .ok_or_else(|| format!("SearchStrategyFlow response bundle missing `{column_name}`"))?;
    if let Some(payloads) = column.as_any().downcast_ref::<BinaryArray>() {
        return decode_binary_response_bundle_payload_batches(payloads, column_name);
    }
    if let Some(payloads) = column.as_any().downcast_ref::<LargeBinaryArray>() {
        return decode_large_binary_response_bundle_payload_batches(payloads, column_name);
    }
    if let Some(payloads) = column.as_any().downcast_ref::<ListArray>() {
        return decode_list_u8_response_bundle_payload_batches(payloads, column_name);
    }
    if let Some(payloads) = column.as_any().downcast_ref::<LargeListArray>() {
        return decode_large_list_u8_response_bundle_payload_batches(payloads, column_name);
    }

    Err(format!(
        "SearchStrategyFlow response bundle `{column_name}` must be Binary, LargeBinary, List(UInt8), or LargeList(UInt8), got {:?}",
        column.data_type()
    ))
}

fn decode_binary_response_bundle_payload_batches(
    payloads: &BinaryArray,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    let mut batches = Vec::new();
    for row_index in 0..payloads.len() {
        if payloads.is_null(row_index) {
            return Err(format!(
                "SearchStrategyFlow response bundle `{column_name}` row {row_index} is null"
            ));
        }
        let payload = payloads.value(row_index);
        if payload.is_empty() {
            return Err(format!(
                "SearchStrategyFlow response bundle `{column_name}` row {row_index} is empty"
            ));
        }
        batches.extend(decode_arrow_ipc_record_batches(payload, column_name)?);
    }
    require_non_empty_batches(batches, column_name)
}

fn decode_large_binary_response_bundle_payload_batches(
    payloads: &LargeBinaryArray,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    let mut batches = Vec::new();
    for row_index in 0..payloads.len() {
        if payloads.is_null(row_index) {
            return Err(format!(
                "SearchStrategyFlow response bundle `{column_name}` row {row_index} is null"
            ));
        }
        let payload = payloads.value(row_index);
        if payload.is_empty() {
            return Err(format!(
                "SearchStrategyFlow response bundle `{column_name}` row {row_index} is empty"
            ));
        }
        batches.extend(decode_arrow_ipc_record_batches(payload, column_name)?);
    }
    require_non_empty_batches(batches, column_name)
}

fn decode_list_u8_response_bundle_payload_batches(
    payloads: &ListArray,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    let mut batches = Vec::new();
    for row_index in 0..payloads.len() {
        let payload = list_u8_payload(payloads, row_index, column_name)?;
        batches.extend(decode_arrow_ipc_record_batches(&payload, column_name)?);
    }
    require_non_empty_batches(batches, column_name)
}

fn decode_large_list_u8_response_bundle_payload_batches(
    payloads: &LargeListArray,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    let mut batches = Vec::new();
    for row_index in 0..payloads.len() {
        let payload = large_list_u8_payload(payloads, row_index, column_name)?;
        batches.extend(decode_arrow_ipc_record_batches(&payload, column_name)?);
    }
    require_non_empty_batches(batches, column_name)
}

fn list_u8_payload(
    payloads: &ListArray,
    row_index: usize,
    column_name: &str,
) -> Result<Vec<u8>, String> {
    if payloads.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response bundle `{column_name}` row {row_index} is null"
        ));
    }
    let values = payloads.value(row_index);
    let bytes = values
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| {
            format!("SearchStrategyFlow response bundle `{column_name}` list values must be UInt8")
        })?;
    u8_payload_from_array(bytes, row_index, column_name)
}

fn large_list_u8_payload(
    payloads: &LargeListArray,
    row_index: usize,
    column_name: &str,
) -> Result<Vec<u8>, String> {
    if payloads.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response bundle `{column_name}` row {row_index} is null"
        ));
    }
    let values = payloads.value(row_index);
    let bytes = values
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| {
            format!("SearchStrategyFlow response bundle `{column_name}` list values must be UInt8")
        })?;
    u8_payload_from_array(bytes, row_index, column_name)
}

fn u8_payload_from_array(
    bytes: &UInt8Array,
    row_index: usize,
    column_name: &str,
) -> Result<Vec<u8>, String> {
    if bytes.is_empty() {
        return Err(format!(
            "SearchStrategyFlow response bundle `{column_name}` row {row_index} is empty"
        ));
    }
    (0..bytes.len())
        .map(|byte_index| {
            if bytes.is_null(byte_index) {
                Err(format!(
                    "SearchStrategyFlow response bundle `{column_name}` row {row_index} byte {byte_index} is null"
                ))
            } else {
                Ok(bytes.value(byte_index))
            }
        })
        .collect()
}

fn has_response_bundle_payload_column(batch: &RecordBatch) -> bool {
    response_bundle_payload_columns()
        .iter()
        .any(|column| batch.column_by_name(column).is_some())
}

fn require_response_bundle_columns(batch: &RecordBatch) -> Result<(), String> {
    let expected_table = WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_RESPONSE_BUNDLE_TABLE;
    if let Some(table_name) = batch.schema().metadata().get("wendao.table")
        && table_name != expected_table
    {
        return Err(format!(
            "SearchStrategyFlow response bundle table metadata must be `{expected_table}` but was `{table_name}`"
        ));
    }
    for column in response_bundle_payload_columns() {
        if batch.column_by_name(column).is_none() {
            return Err(format!(
                "SearchStrategyFlow response bundle missing `{column}`"
            ));
        }
    }
    Ok(())
}

fn response_bundle_payload_columns() -> [&'static str; 4] {
    [
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_CANDIDATES_PAYLOAD_COLUMN,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_TRANSITIONS_PAYLOAD_COLUMN,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_FRONTIER_PAYLOAD_COLUMN,
        WENDAO_GRAPH_SEARCH_STRATEGY_FLOW_STRATEGY_PLANNER_ACTIONS_PAYLOAD_COLUMN,
    ]
}

fn decode_arrow_ipc_record_batches(
    payload: &[u8],
    column: &str,
) -> Result<Vec<RecordBatch>, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None).map_err(|error| {
        format!("SearchStrategyFlow response bundle `{column}` Arrow IPC decode failed: {error}")
    })?;
    let mut batches = Vec::new();
    for batch in reader {
        batches.push(batch.map_err(|error| {
            format!("SearchStrategyFlow response bundle `{column}` Arrow IPC batch failed: {error}")
        })?);
    }
    require_non_empty_batches(batches, column)
}

fn require_non_empty_batches(
    batches: Vec<RecordBatch>,
    column_name: &str,
) -> Result<Vec<RecordBatch>, String> {
    if batches.is_empty() {
        return Err(format!(
            "SearchStrategyFlow response bundle `{column_name}` contained no batches"
        ));
    }
    Ok(batches)
}

fn decode_candidate_batches(
    batches: &[RecordBatch],
) -> Result<Vec<SearchStrategyFlowServiceCandidateRow>, String> {
    batches
        .iter()
        .map(decode_candidate_batch)
        .collect::<Result<Vec<_>, String>>()
        .map(|rows| rows.into_iter().flatten().collect())
}

fn decode_candidate_batch(
    batch: &RecordBatch,
) -> Result<Vec<SearchStrategyFlowServiceCandidateRow>, String> {
    require_columns(
        batch,
        "candidate response",
        &[
            "candidate_id",
            "action",
            "reason",
            "final_score",
            "evidence_coverage",
            "graph_score",
            "authority_score",
            "semantic_score",
            "structural_score",
            "context_cost",
            "blocked",
        ],
    )?;
    (0..batch.num_rows())
        .map(|row_index| {
            Ok(SearchStrategyFlowServiceCandidateRow {
                candidate_id: string_value(batch, "candidate_id", row_index)?.into(),
                action: string_value(batch, "action", row_index)?,
                reason: string_value(batch, "reason", row_index)?,
                final_score: float_value(batch, "final_score", row_index)?,
                evidence_coverage: float_value(batch, "evidence_coverage", row_index)?,
                graph_score: float_value(batch, "graph_score", row_index)?,
                authority_score: float_value(batch, "authority_score", row_index)?,
                semantic_score: float_value(batch, "semantic_score", row_index)?,
                structural_score: float_value(batch, "structural_score", row_index)?,
                context_cost: int_value(batch, "context_cost", row_index)?,
                blocked: bool_value(batch, "blocked", row_index)?,
            })
        })
        .collect()
}

fn decode_frontier_batch(
    batch: &RecordBatch,
) -> Result<Vec<SearchStrategyFlowFrontierRow>, String> {
    require_columns(
        batch,
        "frontier response",
        &[
            "flow_id",
            "frontier_id",
            "candidate_id",
            "revision_id",
            "rank",
            "selected",
            "final_score",
            "action",
            "context_budget",
            "judgement_kind",
        ],
    )?;
    (0..batch.num_rows())
        .map(|row_index| {
            Ok(SearchStrategyFlowFrontierRow {
                flow_id: string_value(batch, "flow_id", row_index)?.into(),
                frontier_id: string_value(batch, "frontier_id", row_index)?.into(),
                candidate_id: string_value(batch, "candidate_id", row_index)?.into(),
                revision_id: string_value(batch, "revision_id", row_index)?.into(),
                rank: int_value(batch, "rank", row_index)?,
                selected: bool_value(batch, "selected", row_index)?,
                final_score: float_value(batch, "final_score", row_index)?,
                action: string_value(batch, "action", row_index)?,
                context_budget: int_value(batch, "context_budget", row_index)?,
                judgement_kind: string_value(batch, "judgement_kind", row_index)?.into(),
            })
        })
        .collect()
}

fn decode_planner_action_batches(
    batches: &[RecordBatch],
) -> Result<Vec<SearchStrategyFlowServicePlannerActionRow>, String> {
    batches
        .iter()
        .map(decode_planner_action_batch)
        .collect::<Result<Vec<_>, String>>()
        .map(|rows| rows.into_iter().flatten().collect())
}

fn decode_planner_action_batch(
    batch: &RecordBatch,
) -> Result<Vec<SearchStrategyFlowServicePlannerActionRow>, String> {
    require_columns(
        batch,
        "planner action response",
        &[
            "action_kind",
            "candidate_id",
            "target_candidate_id",
            "cycle_allowed",
            "requires_llm_judgement",
            "score",
            "context_budget",
            "reason",
        ],
    )?;
    (0..batch.num_rows())
        .map(|row_index| {
            Ok(SearchStrategyFlowServicePlannerActionRow {
                action_kind: string_value(batch, "action_kind", row_index)?.into(),
                candidate_id: string_value(batch, "candidate_id", row_index)?.into(),
                target_candidate_id: string_value(batch, "target_candidate_id", row_index)?.into(),
                cycle_allowed: bool_value(batch, "cycle_allowed", row_index)?,
                requires_llm_judgement: bool_value(batch, "requires_llm_judgement", row_index)?,
                score: float_value(batch, "score", row_index)?,
                context_budget: int_value(batch, "context_budget", row_index)?,
                reason: string_value(batch, "reason", row_index)?,
            })
        })
        .collect()
}

fn require_columns(batch: &RecordBatch, subject: &str, columns: &[&str]) -> Result<(), String> {
    for column in columns {
        if batch.column_by_name(column).is_none() {
            return Err(format!("SearchStrategyFlow {subject} missing `{column}`"));
        }
    }
    Ok(())
}

fn string_value(batch: &RecordBatch, column: &str, row_index: usize) -> Result<String, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("SearchStrategyFlow response missing `{column}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("SearchStrategyFlow response `{column}` must be Utf8"))?;
    if array.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response `{column}` row {row_index} is null"
        ));
    }
    Ok(array.value(row_index).to_owned())
}

fn int_value(batch: &RecordBatch, column: &str, row_index: usize) -> Result<i64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("SearchStrategyFlow response missing `{column}`"))?
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| format!("SearchStrategyFlow response `{column}` must be Int64"))?;
    if array.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response `{column}` row {row_index} is null"
        ));
    }
    Ok(array.value(row_index))
}

fn float_value(batch: &RecordBatch, column: &str, row_index: usize) -> Result<f64, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("SearchStrategyFlow response missing `{column}`"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("SearchStrategyFlow response `{column}` must be Float64"))?;
    if array.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response `{column}` row {row_index} is null"
        ));
    }
    Ok(array.value(row_index))
}

fn bool_value(batch: &RecordBatch, column: &str, row_index: usize) -> Result<bool, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("SearchStrategyFlow response missing `{column}`"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("SearchStrategyFlow response `{column}` must be Boolean"))?;
    if array.is_null(row_index) {
        return Err(format!(
            "SearchStrategyFlow response `{column}` row {row_index} is null"
        ));
    }
    Ok(array.value(row_index))
}
