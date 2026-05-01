//! Lightweight storage errors for Arrow/DataFusion engine surfaces.

use datafusion::error::DataFusionError;
use parquet::errors::ParquetError;
use thiserror::Error;
use tokio::task::JoinError;

/// Errors for local Arrow/DataFusion storage operations.
#[derive(Error, Debug)]
pub enum VectorStoreError {
    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Tokio task join error.
    #[error("Tokio task join error: {0}")]
    JoinError(#[from] JoinError),

    /// Arrow engine error.
    #[error("Arrow engine error: {0}")]
    ArrowEngine(#[from] arrow::error::ArrowError),

    /// Arrow compatibility error.
    #[error("Arrow error: {0}")]
    Arrow(arrow::error::ArrowError),

    /// `DataFusion` error.
    #[error("DataFusion error: {0}")]
    DataFusion(#[from] DataFusionError),

    /// Parquet error.
    #[error("Parquet error: {0}")]
    Parquet(#[from] ParquetError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Table not found.
    #[error("Table not found: {0}")]
    TableNotFound(String),

    /// Invalid embedding dimension.
    #[error("Invalid dimension: expected {expected}, got {actual}")]
    InvalidDimension {
        /// Expected dimension.
        expected: usize,
        /// Actual dimension.
        actual: usize,
    },

    /// Empty dataset.
    #[error("Empty dataset")]
    EmptyDataset,

    /// Invalid embedding dimension (zero or negative).
    #[error("Embedding dimension must be positive")]
    InvalidEmbeddingDimension,

    /// General error with message.
    #[error("{0}")]
    General(String),
}
