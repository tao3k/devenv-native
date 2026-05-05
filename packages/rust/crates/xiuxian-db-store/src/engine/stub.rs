//! Non-vector storage placeholders and Arrow batch compatibility converters.

use std::path::{Path, PathBuf};

use crate::{EngineRecordBatch, LanceRecordBatch, VectorStoreError};

/// Metadata returned by vector-store compaction in explicit vector builds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableInfo {
    /// Number of storage fragments.
    pub fragment_count: usize,
    /// Number of rows in the table.
    pub num_rows: u64,
}

/// Columnar scan options placeholder for non-vector builds.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnarScanOptions {
    /// Optional SQL-like filter.
    pub where_filter: Option<String>,
    /// Optional projected columns. Empty means all columns.
    pub projected_columns: Vec<String>,
    /// Optional scanner batch size.
    pub batch_size: Option<usize>,
    /// Optional fragment read-ahead.
    pub fragment_readahead: Option<usize>,
    /// Optional batch read-ahead.
    pub batch_readahead: Option<usize>,
    /// Optional scan limit.
    pub limit: Option<usize>,
}

/// Non-vector placeholder that keeps default Wendao builds free of `LanceDB`.
#[derive(Debug, Clone)]
pub struct VectorStore {
    base_path: PathBuf,
}

impl VectorStore {
    /// Create a non-vector placeholder store.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty.
    pub async fn new(path: &str, _dimension: Option<usize>) -> Result<Self, VectorStoreError> {
        vector_store_stub_checkpoint().await;
        if path.trim().is_empty() {
            return Err(VectorStoreError::General(
                "vector store path must not be blank".to_string(),
            ));
        }
        Ok(Self {
            base_path: PathBuf::from(path),
        })
    }

    /// Return the table path that an explicit vector-store build would use.
    #[must_use]
    pub fn table_path(&self, table_name: &str) -> PathBuf {
        self.base_path.join(format!("{table_name}.lance"))
    }

    /// Reject compaction in non-vector builds.
    ///
    /// # Errors
    ///
    /// Always returns an error because `LanceDB` is disabled.
    pub async fn compact(&self, _table_name: &str) -> Result<(), VectorStoreError> {
        vector_store_stub_checkpoint().await;
        Err(vector_store_disabled_error())
    }

    /// Reject table inspection in non-vector builds.
    ///
    /// # Errors
    ///
    /// Always returns an error because `LanceDB` is disabled.
    pub async fn get_table_info(&self, _table_name: &str) -> Result<TableInfo, VectorStoreError> {
        vector_store_stub_checkpoint().await;
        Err(vector_store_disabled_error())
    }

    /// Reject vector table replacement in non-vector builds.
    ///
    /// # Errors
    ///
    /// Always returns an error because `LanceDB` is disabled.
    pub async fn replace_record_batches(
        &self,
        _table_name: &str,
        _schema: std::sync::Arc<crate::LanceSchema>,
        _batches: Vec<LanceRecordBatch>,
    ) -> Result<(), VectorStoreError> {
        vector_store_stub_checkpoint().await;
        Err(vector_store_disabled_error())
    }

    /// Reject vector table export in non-vector builds.
    ///
    /// # Errors
    ///
    /// Always returns an error because `LanceDB` is disabled.
    pub async fn write_vector_store_table_to_parquet_file(
        &self,
        _table_name: &str,
        _output_path: &Path,
        _options: ColumnarScanOptions,
    ) -> Result<(), VectorStoreError> {
        vector_store_stub_checkpoint().await;
        Err(vector_store_disabled_error())
    }
}

/// Convert a legacy Lance-named batch into an engine batch.
///
/// The engine-only build uses the same Arrow batch type behind both names.
#[must_use]
pub fn lance_batch_to_engine_batch(batch: &LanceRecordBatch) -> EngineRecordBatch {
    batch.clone()
}

/// Convert legacy Lance-named batches into engine batches.
///
/// The engine-only build uses the same Arrow batch type behind both names.
#[must_use]
pub fn lance_batches_to_engine_batches(batches: &[LanceRecordBatch]) -> Vec<EngineRecordBatch> {
    batches.to_vec()
}

/// Convert an engine batch into a legacy Lance-named batch.
///
/// The engine-only build uses the same Arrow batch type behind both names.
#[must_use]
pub fn engine_batch_to_lance_batch(batch: &EngineRecordBatch) -> LanceRecordBatch {
    batch.clone()
}

/// Convert engine batches into legacy Lance-named batches.
///
/// The engine-only build uses the same Arrow batch type behind both names.
#[must_use]
pub fn engine_batches_to_lance_batches(batches: &[EngineRecordBatch]) -> Vec<LanceRecordBatch> {
    batches.to_vec()
}

async fn vector_store_stub_checkpoint() {
    std::future::ready(()).await;
}

fn vector_store_disabled_error() -> VectorStoreError {
    VectorStoreError::General(
        "LanceDB vector store is disabled; enable the `vector-store` feature for explicit vector/retrieval storage"
            .to_string(),
    )
}
