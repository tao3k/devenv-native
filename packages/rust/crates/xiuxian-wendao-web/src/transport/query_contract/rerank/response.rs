//! Rerank response schema validation for Wendao Flight batches.

#[cfg(feature = "transport")]
use std::collections::HashSet;

#[cfg(feature = "transport")]
use arrow_array::{Float64Array, Int32Array, RecordBatch, StringArray};
#[cfg(feature = "transport")]
use arrow_schema::{DataType, Schema};

/// Canonical rerank response `doc_id` column.
#[cfg(feature = "transport")]
pub const RERANK_RESPONSE_DOC_ID_COLUMN: &str = "doc_id";
/// Canonical rerank response raw vector-score column.
#[cfg(feature = "transport")]
pub const RERANK_RESPONSE_VECTOR_SCORE_COLUMN: &str = "vector_score";
/// Canonical rerank response semantic-score column.
#[cfg(feature = "transport")]
pub const RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN: &str = "semantic_score";
/// Canonical rerank response `final_score` column.
#[cfg(feature = "transport")]
pub const RERANK_RESPONSE_FINAL_SCORE_COLUMN: &str = "final_score";
/// Canonical rerank response `rank` column.
#[cfg(feature = "transport")]
pub const RERANK_RESPONSE_RANK_COLUMN: &str = "rank";

/// Validate the stable rerank response schema.
///
/// # Errors
///
/// Returns an error when the rerank response schema does not match the
/// Rust-owned column set or Arrow types.
#[cfg(feature = "transport")]
pub fn validate_rerank_response_schema(schema: &Schema) -> Result<(), String> {
    let doc_id = schema
        .field_with_name(RERANK_RESPONSE_DOC_ID_COLUMN)
        .map_err(|_| format!("missing rerank response column `{RERANK_RESPONSE_DOC_ID_COLUMN}`"))?;
    if !matches!(doc_id.data_type(), DataType::Utf8) {
        return Err(format!(
            "rerank response column `{RERANK_RESPONSE_DOC_ID_COLUMN}` must be Utf8"
        ));
    }

    let vector_score = schema
        .field_with_name(RERANK_RESPONSE_VECTOR_SCORE_COLUMN)
        .map_err(|_| {
            format!("missing rerank response column `{RERANK_RESPONSE_VECTOR_SCORE_COLUMN}`")
        })?;
    if !matches!(vector_score.data_type(), DataType::Float64) {
        return Err(format!(
            "rerank response column `{RERANK_RESPONSE_VECTOR_SCORE_COLUMN}` must be Float64"
        ));
    }

    let semantic_score = schema
        .field_with_name(RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN)
        .map_err(|_| {
            format!("missing rerank response column `{RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN}`")
        })?;
    if !matches!(semantic_score.data_type(), DataType::Float64) {
        return Err(format!(
            "rerank response column `{RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN}` must be Float64"
        ));
    }

    let final_score = schema
        .field_with_name(RERANK_RESPONSE_FINAL_SCORE_COLUMN)
        .map_err(|_| {
            format!("missing rerank response column `{RERANK_RESPONSE_FINAL_SCORE_COLUMN}`")
        })?;
    if !matches!(final_score.data_type(), DataType::Float64) {
        return Err(format!(
            "rerank response column `{RERANK_RESPONSE_FINAL_SCORE_COLUMN}` must be Float64"
        ));
    }

    let rank = schema
        .field_with_name(RERANK_RESPONSE_RANK_COLUMN)
        .map_err(|_| format!("missing rerank response column `{RERANK_RESPONSE_RANK_COLUMN}`"))?;
    if !matches!(rank.data_type(), DataType::Int32) {
        return Err(format!(
            "rerank response column `{RERANK_RESPONSE_RANK_COLUMN}` must be Int32"
        ));
    }

    Ok(())
}

/// Validate the stable rerank response payload semantics for one decoded batch.
///
/// # Errors
///
/// Returns an error when the rerank response batch contains blank or duplicate
/// document IDs, contains non-finite or out-of-range final scores,
/// or contains non-positive or duplicate rank values.
#[cfg(feature = "transport")]
pub fn validate_rerank_response_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_rerank_response_schema(batch.schema().as_ref())?;
    if batch.num_rows() == 0 {
        return Ok(());
    }

    validate_rerank_response_doc_ids(batch)?;
    validate_rerank_response_score_column(batch, RERANK_RESPONSE_VECTOR_SCORE_COLUMN)?;
    validate_rerank_response_score_column(batch, RERANK_RESPONSE_SEMANTIC_SCORE_COLUMN)?;
    validate_rerank_response_score_column(batch, RERANK_RESPONSE_FINAL_SCORE_COLUMN)?;
    validate_rerank_response_ranks(batch)
}

#[cfg(feature = "transport")]
fn validate_rerank_response_doc_ids(batch: &RecordBatch) -> Result<(), String> {
    let doc_ids = batch
        .column_by_name(RERANK_RESPONSE_DOC_ID_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            format!("rerank response column `{RERANK_RESPONSE_DOC_ID_COLUMN}` must decode as Utf8")
        })?;
    let mut seen_doc_ids = HashSet::new();
    for row_index in 0..batch.num_rows() {
        let doc_id = doc_ids.value(row_index).trim();
        if doc_id.is_empty() {
            return Err(format!(
                "rerank response column `{RERANK_RESPONSE_DOC_ID_COLUMN}` must not contain blank values; row {row_index} is blank"
            ));
        }
        if !seen_doc_ids.insert(doc_id.to_string()) {
            return Err(format!(
                "rerank response column `{RERANK_RESPONSE_DOC_ID_COLUMN}` must be unique across one batch; row {row_index} duplicates `{doc_id}`"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "transport")]
fn validate_rerank_response_score_column(
    batch: &RecordBatch,
    column_name: &'static str,
) -> Result<(), String> {
    let scores = batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| format!("rerank response column `{column_name}` must decode as Float64"))?;
    for row_index in 0..batch.num_rows() {
        let score = scores.value(row_index);
        if !score.is_finite() {
            return Err(format!(
                "rerank response column `{column_name}` must contain finite values; row {row_index} is {score}"
            ));
        }
        if !(0.0..=1.0).contains(&score) {
            return Err(format!(
                "rerank response column `{column_name}` must stay within inclusive range [0.0, 1.0]; row {row_index} is {score}"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "transport")]
fn validate_rerank_response_ranks(batch: &RecordBatch) -> Result<(), String> {
    let ranks = batch
        .column_by_name(RERANK_RESPONSE_RANK_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<Int32Array>())
        .ok_or_else(|| {
            format!("rerank response column `{RERANK_RESPONSE_RANK_COLUMN}` must decode as Int32")
        })?;
    let mut seen_ranks = HashSet::new();
    for row_index in 0..batch.num_rows() {
        let rank = ranks.value(row_index);
        if rank <= 0 {
            return Err(format!(
                "rerank response column `{RERANK_RESPONSE_RANK_COLUMN}` must contain positive values; row {row_index} is {rank}"
            ));
        }
        if !seen_ranks.insert(rank) {
            return Err(format!(
                "rerank response column `{RERANK_RESPONSE_RANK_COLUMN}` must be unique across one batch; row {row_index} duplicates `{rank}`"
            ));
        }
    }
    Ok(())
}
