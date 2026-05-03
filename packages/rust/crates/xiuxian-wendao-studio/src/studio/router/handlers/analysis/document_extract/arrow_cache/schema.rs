use std::sync::Arc;

use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};

pub(super) fn document_resource_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("resourceType", DataType::Utf8, true),
        Field::new("resourcePath", DataType::Utf8, true),
        Field::new("pageIndex", DataType::Int32, true),
        Field::new("caption", DataType::Utf8, true),
        Field::new("content", DataType::Utf8, true),
        Field::new("mimeType", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("elementId", DataType::Utf8, true),
    ]))
}

pub(super) fn document_extract_status_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("jobId", DataType::Utf8, true),
        Field::new("sourcePath", DataType::Utf8, true),
        Field::new("outputDir", DataType::Utf8, true),
        Field::new("contentHash", DataType::Utf8, true),
        Field::new("status", DataType::Utf8, true),
        Field::new("attemptCount", DataType::Int32, true),
        Field::new("createdAtMs", DataType::Int64, true),
        Field::new("startedAtMs", DataType::Int64, true),
        Field::new("finishedAtMs", DataType::Int64, true),
        Field::new("errorMessage", DataType::Utf8, true),
    ]))
}

pub(super) fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}
