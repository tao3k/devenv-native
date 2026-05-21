//! `link_graph::index::search::quantum_fusion::orchestrate::error` owns Wendao quantum fusion orchestrate error behavior.

use crate::link_graph::index::search::quantum_fusion::anchor_batch::QuantumAnchorBatchError;
use arrow::datatypes::DataType;
use arrow::error::ArrowError;
use thiserror::Error;

/// Error returned when Wendao cannot construct scored quantum contexts.
#[derive(Debug, Error)]
pub enum QuantumContextBuildError {
    /// Input batch is missing the configured identifier column.
    #[error("missing required input batch column `{column}`")]
    MissingInputColumn {
        /// Name of the missing column.
        column: String,
    },
    /// Input batch identifier column is not Arrow `Utf8`.
    #[error("input batch column `{column}` must be Utf8, found `{data_type:?}`")]
    InvalidInputUtf8Column {
        /// Name of the offending input column.
        column: String,
        /// Actual Arrow type found in the input batch.
        data_type: DataType,
    },
    /// Input batch score column is not Arrow `Float64`.
    #[error("input batch column `{column}` must be Float64, found `{data_type:?}`")]
    InvalidInputFloat64Column {
        /// Name of the offending input column.
        column: String,
        /// Actual Arrow type found in the input batch.
        data_type: DataType,
    },
    /// Input batch unexpectedly contained a null value.
    #[error("input batch column `{column}` contains null at row {row}")]
    NullInputValue {
        /// Name of the offending input column.
        column: String,
        /// Zero-based row index of the null value.
        row: usize,
    },
    /// Failed to build the Arrow batch fed into the batch scorer.
    #[error("failed to build Arrow batch for quantum-context scoring")]
    BuildScoringBatch(#[source] ArrowError),
    /// The batch scorer rejected the prepared scoring batch.
    #[error("failed to compute quantum-context saliency from scoring batch")]
    ScoreScoringBatch(
        #[source]
        crate::link_graph::index::search::quantum_fusion::scoring::BatchQuantumScorerError,
    ),
    /// The scored batch did not contain the fused saliency column.
    #[error("scored quantum-context batch is missing fused saliency column `{column}`")]
    MissingSaliencyColumn {
        /// Name of the required fused saliency column.
        column: String,
    },
    /// The fused saliency column had the wrong Arrow type.
    #[error(
        "scored quantum-context batch column `{column}` must be Float64, found `{data_type:?}`"
    )]
    InvalidSaliencyColumn {
        /// Name of the offending scored column.
        column: String,
        /// Actual Arrow type found in the scored batch.
        data_type: DataType,
    },
    /// The fused saliency column unexpectedly contained a null value.
    #[error("scored quantum-context batch column `{column}` contains null at row {row}")]
    NullSaliencyValue {
        /// Name of the offending scored column.
        column: String,
        /// Zero-based row index of the null value.
        row: usize,
    },
}

impl From<QuantumAnchorBatchError> for QuantumContextBuildError {
    fn from(error: QuantumAnchorBatchError) -> Self {
        match error {
            QuantumAnchorBatchError::MissingColumn { column } => {
                Self::MissingInputColumn { column }
            }
            QuantumAnchorBatchError::InvalidUtf8Column { column, data_type } => {
                Self::InvalidInputUtf8Column { column, data_type }
            }
            QuantumAnchorBatchError::InvalidFloat64Column { column, data_type } => {
                Self::InvalidInputFloat64Column { column, data_type }
            }
            QuantumAnchorBatchError::NullValue { column, row } => {
                Self::NullInputValue { column, row }
            }
        }
    }
}
