//! Parquet write helpers for search-plane and Lance record batches.

use std::fs::File;
use std::path::Path;

use parquet::arrow::ArrowWriter;

use super::conversion::lance_batches_to_engine_batches;
use crate::{ColumnarScanOptions, LanceRecordBatch, VectorStore, VectorStoreError};

/// Write DataFusion/Arrow-58 batches to a Parquet file.
///
/// # Errors
///
/// Returns an error when the output file cannot be created or the Parquet writer fails.
pub fn write_engine_batches_to_parquet_file(
    output_path: &Path,
    batches: &[arrow::record_batch::RecordBatch],
) -> Result<(), VectorStoreError> {
    let Some(first_batch) = batches.first() else {
        return Ok(());
    };

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = File::create(output_path)?;
    let mut writer = ArrowWriter::try_new(file, first_batch.schema(), None)?;
    for batch in batches {
        writer.write(batch)?;
    }
    writer.close()?;
    Ok(())
}

/// Write Lance/Arrow-57 batches to a Parquet file through the Arrow-58 engine bridge.
///
/// # Errors
///
/// Returns an error when Arrow IPC conversion or Parquet writing fails.
pub fn write_lance_batches_to_parquet_file(
    output_path: &Path,
    batches: &[LanceRecordBatch],
) -> Result<(), VectorStoreError> {
    let engine_batches = lance_batches_to_engine_batches(batches)?;
    write_engine_batches_to_parquet_file(output_path, &engine_batches)
}

impl VectorStore {
    /// Export a Lance-backed columnar table into a Parquet file for `DataFusion` reads.
    ///
    /// # Errors
    ///
    /// Returns an error when the source table scan, Arrow conversion, or Parquet write fails.
    pub async fn write_vector_store_table_to_parquet_file(
        &self,
        table_name: &str,
        output_path: &Path,
        options: ColumnarScanOptions,
    ) -> Result<(), VectorStoreError> {
        let batches = self.scan_record_batches(table_name, options).await?;
        write_lance_batches_to_parquet_file(output_path, &batches)
    }
}
