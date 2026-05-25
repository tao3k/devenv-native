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
#[path = "../tests/unit/arrow_metadata.rs"]
mod tests;
