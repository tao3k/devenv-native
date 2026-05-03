//! Rerank scoring implementation for Wendao Flight batches.

#[cfg(feature = "transport")]
use arrow_array::{FixedSizeListArray, Float32Array, RecordBatch, StringArray};

#[cfg(feature = "transport")]
use super::request::{
    RERANK_REQUEST_DOC_ID_COLUMN, RERANK_REQUEST_EMBEDDING_COLUMN,
    RERANK_REQUEST_QUERY_EMBEDDING_COLUMN, RERANK_REQUEST_VECTOR_SCORE_COLUMN,
    fixed_size_list_row_values, validate_rerank_request_batch,
};
#[cfg(feature = "transport")]
use super::types::{RerankScoreWeights, RerankScoredCandidate};

/// Score one validated rerank request batch with the shared Rust-owned rerank rule.
///
/// The current stable rule blends the inbound vector score with semantic cosine
/// similarity between `embedding` and `query_embedding`:
///
/// - `semantic_score = (cosine_similarity + 1.0) / 2.0`
/// - `final_score = 0.4 * vector_score + 0.6 * semantic_score`
///
/// # Errors
///
/// Returns an error when the request batch fails validation or when any
/// embedding/query vector has zero norm.
#[cfg(feature = "transport")]
pub fn score_rerank_request_batch(
    batch: &RecordBatch,
    expected_dimension: usize,
) -> Result<Vec<RerankScoredCandidate>, String> {
    score_rerank_request_batch_with_weights(
        batch,
        expected_dimension,
        RerankScoreWeights::default(),
    )
}

/// Score one validated rerank request batch with explicit transport-owned
/// rerank weights.
///
/// # Errors
///
/// Returns an error when the request batch fails validation, when any
/// embedding/query vector has zero norm, or when the weights are invalid.
#[cfg(feature = "transport")]
pub fn score_rerank_request_batch_with_weights(
    batch: &RecordBatch,
    expected_dimension: usize,
    weights: RerankScoreWeights,
) -> Result<Vec<RerankScoredCandidate>, String> {
    validate_rerank_request_batch(batch, expected_dimension)?;
    let weights =
        RerankScoreWeights::new(weights.vector_weight, weights.semantic_weight)?.normalized();

    let doc_ids = batch
        .column_by_name(RERANK_REQUEST_DOC_ID_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            format!("rerank request column `{RERANK_REQUEST_DOC_ID_COLUMN}` must decode as Utf8")
        })?;
    let vector_scores = batch
        .column_by_name(RERANK_REQUEST_VECTOR_SCORE_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>())
        .ok_or_else(|| {
            format!(
                "rerank request column `{RERANK_REQUEST_VECTOR_SCORE_COLUMN}` must decode as Float32"
            )
        })?;
    let embeddings = batch
        .column_by_name(RERANK_REQUEST_EMBEDDING_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
        .ok_or_else(|| {
            format!(
                "rerank request column `{RERANK_REQUEST_EMBEDDING_COLUMN}` must decode as FixedSizeList<Float32>"
            )
        })?;
    let query_embeddings = batch
        .column_by_name(RERANK_REQUEST_QUERY_EMBEDDING_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
        .ok_or_else(|| {
            format!(
                "rerank request column `{RERANK_REQUEST_QUERY_EMBEDDING_COLUMN}` must decode as FixedSizeList<Float32>"
            )
        })?;

    let mut scored_candidates = Vec::with_capacity(batch.num_rows());
    for row_index in 0..batch.num_rows() {
        let embedding = fixed_size_list_row_values(embeddings, row_index)?;
        let query_embedding = fixed_size_list_row_values(query_embeddings, row_index)?;
        let cosine = cosine_similarity(&embedding, &query_embedding, row_index)?;
        let vector_score = f64::from(vector_scores.value(row_index));
        let semantic_score = f64::midpoint(cosine, 1.0);
        let final_score =
            weights.vector_weight * vector_score + weights.semantic_weight * semantic_score;
        scored_candidates.push(RerankScoredCandidate {
            doc_id: doc_ids.value(row_index).to_string(),
            vector_score,
            semantic_score,
            final_score,
        });
    }

    Ok(scored_candidates)
}

#[cfg(feature = "transport")]
fn cosine_similarity(left: &[f32], right: &[f32], row_index: usize) -> Result<f64, String> {
    let left_norm = left
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if left_norm == 0.0 {
        return Err(format!(
            "rerank request column `{RERANK_REQUEST_EMBEDDING_COLUMN}` must not contain zero-norm vectors; row {row_index} is zero"
        ));
    }

    let right_norm = right
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if right_norm == 0.0 {
        return Err(format!(
            "rerank request column `{RERANK_REQUEST_QUERY_EMBEDDING_COLUMN}` must not contain zero-norm vectors; row {row_index} is zero"
        ));
    }

    let dot = left
        .iter()
        .zip(right.iter())
        .map(|(left_value, right_value)| f64::from(*left_value) * f64::from(*right_value))
        .sum::<f64>();
    Ok((dot / (left_norm * right_norm)).clamp(-1.0, 1.0))
}
