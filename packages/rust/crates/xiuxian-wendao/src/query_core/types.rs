//! `query_core::types` owns Wendao query core types behavior.

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use thiserror::Error;

/// Logical operator kinds supported by the Phase-1 query core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoOperatorKind {
    /// Retrieval-first candidate search.
    VectorSearch,
    /// Graph-neighbor expansion.
    GraphNeighbors,
    /// Narrow-column filtering before payload hydration.
    ColumnMask,
    /// Payload hydration and projection.
    PayloadFetch,
}

/// Execution backend kinds surfaced through explain events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WendaoBackendKind {
    /// Search-plane-backed retrieval execution.
    SearchPlaneBackend,
    /// Link-graph-backed neighbor execution.
    LinkGraphBackend,
    /// Native query-core narrow-mask execution.
    QueryCoreMask,
    /// Retrieval-side payload projection adapter.
    VectorRetrievalAdapter,
}

/// Arrow-native relation wrapper shared between query-core operators.
#[derive(Debug, Clone)]
pub struct WendaoRelation {
    schema: SchemaRef,
    batches: Vec<RecordBatch>,
}

impl WendaoRelation {
    /// Build a relation from a schema and owned batches.
    #[must_use]
    pub fn new(schema: SchemaRef, batches: Vec<RecordBatch>) -> Self {
        Self { schema, batches }
    }

    /// Borrow the relation schema.
    #[must_use]
    pub fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    /// Borrow the relation batches.
    #[must_use]
    pub fn batches(&self) -> &[RecordBatch] {
        &self.batches
    }

    /// Count rows across all owned batches.
    #[must_use]
    pub fn row_count(&self) -> usize {
        self.batches.iter().map(RecordBatch::num_rows).sum()
    }
}

/// Errors emitted by the Phase-1 query core.
#[derive(Debug, Error)]
pub enum WendaoQueryCoreError {
    /// A required backend adapter was not attached to the execution context.
    #[error("missing query-core backend: {0}")]
    MissingBackend(&'static str),
    /// Retrieval helper failure from the storage facade.
    #[error("vector backend error: {0}")]
    Vector(#[from] xiuxian_db_store::VectorStoreError),
    /// Arrow batch construction or projection failure.
    #[error("arrow error: {0}")]
    Arrow(#[from] arrow::error::ArrowError),
    /// Adapter-specific backend error.
    #[error("query-core backend error: {0}")]
    Backend(String),
    /// Invalid input relation or projection request.
    #[error("invalid relation: {0}")]
    InvalidRelation(String),
}
