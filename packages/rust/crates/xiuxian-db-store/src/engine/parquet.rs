use std::fs::File;
use std::path::Path;

use parquet::arrow::ArrowWriter;

use crate::{EngineRecordBatch, LanceRecordBatch, VectorStoreError};

/// Write Arrow engine batches to a Parquet file.
///
/// # Errors
///
/// Returns an error when the output file cannot be created or the Parquet
/// writer fails.
pub fn write_engine_batches_to_parquet_file(
    output_path: &Path,
    batches: &[EngineRecordBatch],
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

/// Write legacy Lance-named Arrow batches to a Parquet file.
///
/// # Errors
///
/// Returns an error when the output file cannot be created or the Parquet
/// writer fails.
pub fn write_lance_batches_to_parquet_file(
    output_path: &Path,
    batches: &[LanceRecordBatch],
) -> Result<(), VectorStoreError> {
    write_engine_batches_to_parquet_file(output_path, batches)
}
