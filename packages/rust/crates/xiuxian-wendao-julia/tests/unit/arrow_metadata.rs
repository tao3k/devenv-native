use std::sync::Arc;

use arrow::array::StringArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::attach_record_batch_metadata;

#[test]
fn attach_record_batch_metadata_merges_existing_entries() {
    let schema = Arc::new(Schema::new_with_metadata(
        vec![Field::new("name", DataType::Utf8, false)],
        [("existing".to_string(), "one".to_string())]
            .into_iter()
            .collect(),
    ));
    let batch = RecordBatch::try_new(schema, vec![Arc::new(StringArray::from(vec!["julia"]))])
        .unwrap_or_else(|error| panic!("build batch: {error}"));

    let updated =
        attach_record_batch_metadata(&batch, [("trace_id", "trace-123"), ("existing", "two")])
            .unwrap_or_else(|error| panic!("attach metadata: {error}"));

    assert_eq!(
        updated
            .schema()
            .metadata()
            .get("trace_id")
            .map(String::as_str),
        Some("trace-123")
    );
    assert_eq!(
        updated
            .schema()
            .metadata()
            .get("existing")
            .map(String::as_str),
        Some("two")
    );
}
