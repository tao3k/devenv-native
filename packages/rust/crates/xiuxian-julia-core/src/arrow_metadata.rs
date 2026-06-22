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
    xiuxian_db_store::attach_record_batch_metadata(batch, metadata)
}

#[cfg(test)]
#[path = "../tests/unit/arrow_metadata.rs"]
mod tests;
