//! Rerank request schema validation for Wendao Flight batches.

#[cfg(feature = "transport")]
use std::collections::HashSet;

#[cfg(feature = "transport")]
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};
#[cfg(feature = "transport")]
use arrow_schema::{DataType, Schema};

/// Canonical rerank request `doc_id` column.
#[cfg(feature = "transport")]
pub const RERANK_REQUEST_DOC_ID_COLUMN: &str = "doc_id";
/// Canonical rerank request `vector_score` column.
#[cfg(feature = "transport")]
pub const RERANK_REQUEST_VECTOR_SCORE_COLUMN: &str = "vector_score";
/// Canonical rerank request `embedding` column.
#[cfg(feature = "transport")]
pub const RERANK_REQUEST_EMBEDDING_COLUMN: &str = "embedding";
/// Canonical rerank request `query_embedding` column.
#[cfg(feature = "transport")]
pub const RERANK_REQUEST_QUERY_EMBEDDING_COLUMN: &str = "query_embedding";

/// Validate the stable rerank request schema for one expected embedding dimension.
///
/// # Errors
///
/// Returns an error when the rerank request schema does not match the stable
/// Rust-owned column set or Arrow types.
#[cfg(feature = "transport")]
pub fn validate_rerank_request_schema(
    schema: &Schema,
    expected_dimension: usize,
) -> Result<(), String> {
    let doc_id = schema
        .field_with_name(RERANK_REQUEST_DOC_ID_COLUMN)
        .map_err(|_| format!("missing rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}`"))?;
    if !matches!(doc_id.data_type(), DataType::Utf8) {
        return Err(format!(
            "rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}` must be Utf8"
        ));
    }

    let vector_score = schema
        .field_with_name(RERANK_REQUEST_VECTOR_SCORE_COLUMN)
        .map_err(|_| {
            format!("missing rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}`")
        })?;
    if !matches!(vector_score.data_type(), DataType::Float32) {
        return Err(format!(
            "rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}` must be Float32"
        ));
    }

    validate_embedding_field(schema, RERANK_REQUEST_EMBEDDING_COLUMN, expected_dimension)?;
    validate_embedding_field(
        schema,
        RERANK_REQUEST_QUERY_EMBEDDING_COLUMN,
        expected_dimension,
    )?;
    Ok(())
}

/// Validate the stable rerank request payload semantics for one decoded batch.
///
/// # Errors
///
/// Returns an error when the rerank request batch is empty, contains blank
/// document IDs, contains duplicate document IDs, contains non-finite or
/// out-of-range vector scores, or carries drifted `query_embedding` values
/// across rows.
#[cfg(feature = "transport")]
pub fn validate_rerank_request_batch(
    batch: &RecordBatch,
    expected_dimension: usize,
) -> Result<(), String> {
    validate_rerank_request_schema(batch.schema().as_ref(), expected_dimension)?;
    if batch.num_rows() == 0 {
        return Err("rerank request batch must contain at least one row".to_string());
    }

    validate_rerank_request_doc_ids(batch)?;
    validate_rerank_request_vector_scores(batch)?;
    validate_rerank_request_query_embeddings(batch)
}

#[cfg(feature = "transport")]
fn validate_rerank_request_doc_ids(batch: &RecordBatch) -> Result<(), String> {
    let doc_ids = batch
        .column_by_name(RERANK_REQUEST_DOC_ID_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            format!("rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}` must decode as Utf8")
        })?;
    let mut seen_doc_ids = HashSet::new();
    for row_index in 0..batch.num_rows() {
        let doc_id = doc_ids.value(row_index).trim();
        if doc_id.is_empty() {
            return Err(format!(
                "rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}` must not contain blank values; row {row_index} is blank"
            ));
        }
        if !seen_doc_ids.insert(doc_id.to_string()) {
            return Err(format!(
                "rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}` must be unique across one batch; row {row_index} duplicates `{doc_id}`"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "transport")]
fn validate_rerank_request_vector_scores(batch: &RecordBatch) -> Result<(), String> {
    let vector_scores = batch
        .column_by_name(RERANK_REQUEST_VECTOR_SCORE_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>())
        .ok_or_else(|| {
            format!(
                "rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}` must decode as Float32"
            )
        })?;
    for row_index in 0..batch.num_rows() {
        let score = vector_scores.value(row_index);
        if !score.is_finite() {
            return Err(format!(
                "rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}` must contain finite values; row {row_index} is {score}"
            ));
        }
        if !(0.0..=1.0).contains(&score) {
            return Err(format!(
                "rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}` must stay within inclusive range [0.0, 1.0]; row {row_index} is {score}"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "transport")]
fn validate_rerank_request_query_embeddings(batch: &RecordBatch) -> Result<(), String> {
    let query_embeddings = batch
        .column_by_name(RERANK_REQUEST_QUERY_EMBEDDING_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
        .ok_or_else(|| {
            format!(
                "rerank request column `{RERANK_REQUEST_QUERY_EMBEDDING_COLUMN}` must decode as FixedSizeList<Float32>"
            )
        })?;
    let first_query_embedding = fixed_size_list_row_values(query_embeddings, 0)?;
    for row_index in 1..batch.num_rows() {
        let row_query_embedding = fixed_size_list_row_values(query_embeddings, row_index)?;
        if row_query_embedding != first_query_embedding {
            return Err(format!(
                "rerank request column `{RERANK_REQUEST_QUERY_EMBEDDING_COLUMN}` must remain stable across all rows; row {row_index} differs from row 0"
            ));
        }
    }

    Ok(())
}

#[cfg(feature = "transport")]
fn validate_embedding_field(
    schema: &Schema,
    field_name: &str,
    expected_dimension: usize,
) -> Result<(), String> {
    let field = schema
        .field_with_name(field_name)
        .map_err(|_| format!("missing rerank request column `{field_name}`"))?;
    match field.data_type() {
        DataType::FixedSizeList(inner_field, dimension)
            if matches!(inner_field.data_type(), DataType::Float32)
                && usize::try_from(*dimension).ok() == Some(expected_dimension) =>
        {
            Ok(())
        }
        DataType::FixedSizeList(inner_field, dimension)
            if matches!(inner_field.data_type(), DataType::Float32) =>
        {
            Err(format!(
                "rerank request column `{field_name}` must use dimension {expected_dimension}, got {dimension}"
            ))
        }
        _ => Err(format!(
            "rerank request column `{field_name}` must be FixedSizeList<Float32>"
        )),
    }
}

#[cfg(feature = "transport")]
pub(super) fn fixed_size_list_row_values(
    array: &FixedSizeListArray,
    row_index: usize,
) -> Result<Vec<f32>, String> {
    let row = array.value(row_index);
    let values = row.as_any().downcast_ref::<Float32Array>().ok_or_else(|| {
        "rerank request fixed-size-list values must decode as Float32".to_string()
    })?;
    Ok((0..values.len()).map(|index| values.value(index)).collect())
}
