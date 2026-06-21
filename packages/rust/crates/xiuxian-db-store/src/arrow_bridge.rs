//! Arrow batch compatibility aliases for transport and storage boundaries.

pub use arrow::array::builder::{
    ListBuilder as LanceListBuilder, StringBuilder as LanceStringBuilder,
};
pub use arrow::array::{
    Array as LanceArray, ArrayRef as LanceArrayRef, BooleanArray as LanceBooleanArray,
    FixedSizeListArray as LanceFixedSizeListArray, Float32Array as LanceFloat32Array,
    Float64Array as LanceFloat64Array, Int32Array as LanceInt32Array, ListArray as LanceListArray,
    RecordBatch as LanceRecordBatch, StringArray as LanceStringArray,
    UInt32Array as LanceUInt32Array, UInt64Array as LanceUInt64Array,
};
pub use arrow::datatypes::{DataType as LanceDataType, Field as LanceField, Schema as LanceSchema};
pub use arrow::record_batch::RecordBatch as EngineRecordBatch;

/// Convert a legacy Lance-named batch into an engine batch.
///
/// Non-vector builds use the same Arrow batch type behind both names.
#[must_use]
pub fn lance_batch_to_engine_batch(batch: &LanceRecordBatch) -> EngineRecordBatch {
    batch.clone()
}

/// Convert legacy Lance-named batches into engine batches.
///
/// Non-vector builds use the same Arrow batch type behind both names.
#[must_use]
pub fn lance_batches_to_engine_batches(batches: &[LanceRecordBatch]) -> Vec<EngineRecordBatch> {
    batches.to_vec()
}

/// Convert an engine batch into a legacy Lance-named batch.
///
/// Non-vector builds use the same Arrow batch type behind both names.
#[must_use]
pub fn engine_batch_to_lance_batch(batch: &EngineRecordBatch) -> LanceRecordBatch {
    batch.clone()
}

/// Convert engine batches into legacy Lance-named batches.
///
/// Non-vector builds use the same Arrow batch type behind both names.
#[must_use]
pub fn engine_batches_to_lance_batches(batches: &[EngineRecordBatch]) -> Vec<LanceRecordBatch> {
    batches.to_vec()
}
