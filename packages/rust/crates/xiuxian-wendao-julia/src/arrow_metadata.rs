use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::Schema;
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;

/// Attach or overwrite schema metadata on a `RecordBatch`.
///
/// Existing schema metadata is preserved unless a provided key overwrites it.
///
/// # Errors
///
/// Returns [`ArrowError`] when the batch cannot be rebuilt with the merged
/// schema metadata.
pub(crate) fn attach_record_batch_metadata<K, V, I>(
    batch: &RecordBatch,
    metadata: I,
) -> Result<RecordBatch, ArrowError>
where
    K: Into<String>,
    V: Into<String>,
    I: IntoIterator<Item = (K, V)>,
{
    let mut merged: HashMap<String, String> = batch.schema().metadata().clone();
    merged.extend(
        metadata
            .into_iter()
            .map(|(key, value)| (key.into(), value.into())),
    );

    let schema = Arc::new(Schema::new_with_metadata(
        batch.schema().fields().clone(),
        merged,
    ));
    RecordBatch::try_new(schema, batch.columns().to_vec())
}

#[cfg(test)]
mod tests {
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
}
