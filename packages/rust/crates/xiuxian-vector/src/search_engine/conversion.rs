//! Arrow-native conversion between engine-native and Lance-native record batches.

use arrow::record_batch::RecordBatch as EngineRecordBatch;

use crate::{LanceRecordBatch, VectorStoreError};

/// Convert a Lance-owned record batch into a workspace Arrow/DataFusion record batch.
///
/// # Errors
///
/// This API is fallible for compatibility with older call sites. With Lance 4
/// and Arrow 58, the batch type is shared and conversion is a clone.
pub fn lance_batch_to_engine_batch(
    batch: &LanceRecordBatch,
) -> Result<EngineRecordBatch, VectorStoreError> {
    Ok(batch.clone())
}

/// Convert multiple Lance-owned batches into workspace Arrow/DataFusion batches.
///
/// # Errors
///
/// This API is fallible for compatibility with older call sites. With Lance 4
/// and Arrow 58, the batch type is shared and conversion clones the batch list.
pub fn lance_batches_to_engine_batches(
    batches: &[LanceRecordBatch],
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    Ok(batches.to_vec())
}

/// Convert a workspace Arrow/DataFusion batch into a Lance-owned record batch.
///
/// # Errors
///
/// This API is fallible for compatibility with older call sites. With Lance 4
/// and Arrow 58, the batch type is shared and conversion is a clone.
pub fn engine_batch_to_lance_batch(
    batch: &EngineRecordBatch,
) -> Result<LanceRecordBatch, VectorStoreError> {
    Ok(batch.clone())
}

/// Convert multiple workspace Arrow/DataFusion batches into Lance-owned batches.
///
/// # Errors
///
/// This API is fallible for compatibility with older call sites. With Lance 4
/// and Arrow 58, the batch type is shared and conversion clones the batch list.
pub fn engine_batches_to_lance_batches(
    batches: &[EngineRecordBatch],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    Ok(batches.to_vec())
}
