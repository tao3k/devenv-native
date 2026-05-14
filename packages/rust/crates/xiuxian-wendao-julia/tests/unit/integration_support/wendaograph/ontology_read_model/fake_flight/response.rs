use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

pub(super) fn ontology_quality_response_batch() -> RecordBatch {
    RecordBatch::try_new(
        Arc::new(Schema::new(vec![
            Field::new("check_id", DataType::Utf8, false),
            Field::new("status", DataType::Utf8, false),
        ])),
        vec![
            Arc::new(StringArray::from(vec!["object_graph_component_count"])),
            Arc::new(StringArray::from(vec!["pass"])),
        ],
    )
    .unwrap_or_else(|error| panic!("ontology quality response batch should build: {error}"))
}
