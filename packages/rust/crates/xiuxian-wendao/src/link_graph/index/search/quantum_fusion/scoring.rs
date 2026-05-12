//! `link_graph::index::search::quantum_fusion::scoring` owns Wendao search quantum fusion scoring behavior.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{Array, Float64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use thiserror::Error;

use crate::link_graph::models::QuantumFusionOptions;

/// Output column appended by [`BatchQuantumScorer`].
pub const QUANTUM_SALIENCY_COLUMN: &str = "quantum_saliency";

/// Arrow-native scorer that fuses semantic and topology scores in one batch pass.
#[derive(Debug, Clone)]
pub struct BatchQuantumScorer {
    options: QuantumFusionOptions,
}

struct BatchScoreColumns<'a> {
    ids: &'a StringArray,
    similarities: &'a Float64Array,
}

impl BatchQuantumScorer {
    /// Create a new batch scorer with normalized fusion options.
    #[must_use]
    pub fn new(options: &QuantumFusionOptions) -> Self {
        Self {
            options: options.normalized(),
        }
    }

    /// Fuse semantic and topology scores for every row in an Arrow `RecordBatch`.
    ///
    /// The `ppr_map` values are expected to be pre-normalized topology saliency
    /// scores keyed by the same identifiers stored in `id_col`.
    ///
    /// # Errors
    ///
    /// Returns [`BatchQuantumScorerError`] when required columns are missing,
    /// column types do not match the expected Arrow layout, a required value is
    /// null, or the fused output batch cannot be constructed.
    pub fn score_batch(
        &self,
        batch: &RecordBatch,
        ppr_map: &HashMap<String, f64>,
        id_col: &str,
        sim_col: &str,
    ) -> Result<RecordBatch, BatchQuantumScorerError> {
        let score_columns = score_columns(batch, id_col, sim_col)?;
        let fused_scores = self.fused_scores(batch, &score_columns, ppr_map, id_col, sim_col)?;
        let schema = score_batch_schema(batch);
        let columns = score_batch_columns(batch, fused_scores);
        RecordBatch::try_new(schema, columns).map_err(BatchQuantumScorerError::Arrow)
    }

    fn fused_scores(
        &self,
        batch: &RecordBatch,
        score_columns: &BatchScoreColumns<'_>,
        ppr_map: &HashMap<String, f64>,
        id_col: &str,
        sim_col: &str,
    ) -> Result<Vec<f64>, BatchQuantumScorerError> {
        (0..batch.num_rows())
            .map(|row| self.fused_score_for_row(score_columns, ppr_map, id_col, sim_col, row))
            .collect()
    }

    fn fused_score_for_row(
        &self,
        score_columns: &BatchScoreColumns<'_>,
        ppr_map: &HashMap<String, f64>,
        id_col: &str,
        sim_col: &str,
        row: usize,
    ) -> Result<f64, BatchQuantumScorerError> {
        ensure_non_null(score_columns.ids, id_col, row)?;
        ensure_non_null(score_columns.similarities, sim_col, row)?;
        let doc_id = score_columns.ids.value(row);
        let semantic_score = score_columns.similarities.value(row);
        let topology_score = ppr_map.get(doc_id).copied().unwrap_or(0.0);
        Ok(fuse_saliency_score(
            semantic_score,
            topology_score,
            &self.options,
        ))
    }
}

fn score_columns<'a>(
    batch: &'a RecordBatch,
    id_col: &str,
    sim_col: &str,
) -> Result<BatchScoreColumns<'a>, BatchQuantumScorerError> {
    Ok(BatchScoreColumns {
        ids: utf8_column(batch, id_col)?,
        similarities: float64_column(batch, sim_col)?,
    })
}

fn utf8_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
) -> Result<&'a StringArray, BatchQuantumScorerError> {
    let array =
        batch
            .column_by_name(column)
            .ok_or_else(|| BatchQuantumScorerError::MissingColumn {
                column: column.to_string(),
            })?;
    array.as_any().downcast_ref::<StringArray>().ok_or_else(|| {
        BatchQuantumScorerError::InvalidUtf8Column {
            column: column.to_string(),
            data_type: array.data_type().clone(),
        }
    })
}

fn float64_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
) -> Result<&'a Float64Array, BatchQuantumScorerError> {
    let array =
        batch
            .column_by_name(column)
            .ok_or_else(|| BatchQuantumScorerError::MissingColumn {
                column: column.to_string(),
            })?;
    array
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| BatchQuantumScorerError::InvalidFloat64Column {
            column: column.to_string(),
            data_type: array.data_type().clone(),
        })
}

fn ensure_non_null(
    array: &dyn Array,
    column: &str,
    row: usize,
) -> Result<(), BatchQuantumScorerError> {
    if array.is_null(row) {
        return Err(BatchQuantumScorerError::NullValue {
            column: column.to_string(),
            row,
        });
    }
    Ok(())
}

fn score_batch_schema(batch: &RecordBatch) -> Arc<Schema> {
    let mut fields = batch
        .schema_ref()
        .fields()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    fields.push(Arc::new(Field::new(
        QUANTUM_SALIENCY_COLUMN,
        DataType::Float64,
        false,
    )));
    Arc::new(Schema::new_with_metadata(
        fields,
        batch.schema_ref().metadata().clone(),
    ))
}

fn score_batch_columns(batch: &RecordBatch, fused_scores: Vec<f64>) -> Vec<Arc<dyn Array>> {
    let fused_array: Arc<dyn Array> = Arc::new(Float64Array::from(fused_scores));
    let mut columns = batch.columns().to_vec();
    columns.push(fused_array);
    columns
}

/// Error returned when Arrow-native batch scoring cannot be completed.
#[derive(Debug, Error)]
pub enum BatchQuantumScorerError {
    /// Required input column is missing from the batch schema.
    #[error("missing required batch column `{column}`")]
    MissingColumn {
        /// Name of the missing column.
        column: String,
    },
    /// Input id column is not Arrow `Utf8`.
    #[error("batch column `{column}` must be Utf8, found `{data_type:?}`")]
    InvalidUtf8Column {
        /// Name of the offending column.
        column: String,
        /// Actual Arrow data type found in the batch.
        data_type: DataType,
    },
    /// Input similarity column is not Arrow `Float64`.
    #[error("batch column `{column}` must be Float64, found `{data_type:?}`")]
    InvalidFloat64Column {
        /// Name of the offending column.
        column: String,
        /// Actual Arrow data type found in the batch.
        data_type: DataType,
    },
    /// Required cell is null.
    #[error("batch column `{column}` contains null at row {row}")]
    NullValue {
        /// Name of the offending column.
        column: String,
        /// Zero-based row index carrying the null value.
        row: usize,
    },
    /// Arrow failed to construct the fused batch.
    #[error("failed to construct fused RecordBatch: {0}")]
    Arrow(ArrowError),
}

pub(in crate::link_graph::index::search::quantum_fusion) fn fuse_saliency_score(
    vector_score: f64,
    topology_score: f64,
    options: &QuantumFusionOptions,
) -> f64 {
    let alpha = options.alpha.clamp(0.0, 1.0);
    let semantic = vector_score.clamp(0.0, 1.0);
    let topology = topology_score.clamp(0.0, 1.0);
    alpha * semantic + (1.0 - alpha) * topology
}

pub(in crate::link_graph::index::search::quantum_fusion) fn distance_to_score(
    distance: f64,
) -> f64 {
    1.0 / (1.0 + distance.max(0.0))
}

pub(in crate::link_graph::index::search::quantum_fusion) fn topology_score_from_ranked(
    ranked: &[(String, usize, f64)],
    related_limit: usize,
) -> f64 {
    ranked
        .iter()
        .take(related_limit.max(1))
        .map(|(_, _, score)| score.max(0.0))
        .sum::<f64>()
        .clamp(0.0, 1.0)
}

#[cfg(test)]
#[path = "../../../../../tests/unit/link_graph/index/search/quantum_fusion/scoring.rs"]
mod tests;
