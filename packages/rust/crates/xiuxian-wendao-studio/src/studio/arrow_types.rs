//! Studio-local Arrow aliases for Flight payload construction.

#[cfg(test)]
pub(crate) use arrow::array::Array as LanceArray;
pub(crate) use arrow::array::{
    ArrayRef as LanceArrayRef, BooleanArray as LanceBooleanArray,
    Float32Array as LanceFloat32Array, Float64Array as LanceFloat64Array,
    Int32Array as LanceInt32Array, StringArray as LanceStringArray,
    UInt64Array as LanceUInt64Array,
};
pub(crate) use arrow::datatypes::{
    DataType as LanceDataType, Field as LanceField, Schema as LanceSchema,
};
pub(crate) use arrow::record_batch::RecordBatch as LanceRecordBatch;
