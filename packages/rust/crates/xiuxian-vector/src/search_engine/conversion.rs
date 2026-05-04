//! Arrow-native conversion between engine-native and Lance-native record batches.

use arrow::record_batch::RecordBatch as EngineRecordBatch;

use crate::LanceRecordBatch;

/// Convert a Lance-owned record batch into a workspace Arrow/DataFusion record batch.
///
/// With Lance and Arrow 58, the batch type is shared and conversion is a clone.
#[must_use]
pub fn lance_batch_to_engine_batch(batch: &LanceRecordBatch) -> EngineRecordBatch {
    batch.clone()
}

/// Convert multiple Lance-owned batches into workspace Arrow/DataFusion batches.
///
/// With Lance and Arrow 58, the batch type is shared and conversion clones the batch list.
#[must_use]
pub fn lance_batches_to_engine_batches(batches: &[LanceRecordBatch]) -> Vec<EngineRecordBatch> {
    batches.to_vec()
}

/// Convert a workspace Arrow/DataFusion batch into a Lance-owned record batch.
///
/// With Lance and Arrow 58, the batch type is shared and conversion is a clone.
#[must_use]
pub fn engine_batch_to_lance_batch(batch: &EngineRecordBatch) -> LanceRecordBatch {
    batch.clone()
}

/// Convert multiple workspace Arrow/DataFusion batches into Lance-owned batches.
///
/// With Lance and Arrow 58, the batch type is shared and conversion clones the batch list.
#[must_use]
pub fn engine_batches_to_lance_batches(batches: &[EngineRecordBatch]) -> Vec<LanceRecordBatch> {
    batches.to_vec()
}
