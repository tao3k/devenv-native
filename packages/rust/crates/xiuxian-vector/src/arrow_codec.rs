//! Arrow IPC encoding and metadata helpers shared across vector interfaces.

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use arrow::datatypes::Schema;
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;

const TRACE_ID_METADATA_KEY: &str = "trace_id";

/// Encode a single Arrow `RecordBatch` into IPC stream bytes.
///
/// # Errors
///
/// Returns [`ArrowError`] when Arrow IPC stream construction fails.
pub fn encode_record_batch_ipc(batch: &RecordBatch) -> Result<Vec<u8>, ArrowError> {
    encode_record_batches_ipc(std::slice::from_ref(batch))
}

/// Encode one or more Arrow `RecordBatch` values into IPC stream bytes.
///
/// # Errors
///
/// Returns [`ArrowError`] when the batch list is empty or Arrow IPC stream
/// construction fails.
pub fn encode_record_batches_ipc(batches: &[RecordBatch]) -> Result<Vec<u8>, ArrowError> {
    let Some(first_batch) = batches.first() else {
        return Err(ArrowError::InvalidArgumentError(
            "Arrow IPC encoding requires at least one RecordBatch".to_string(),
        ));
    };

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = StreamWriter::try_new(&mut buffer, first_batch.schema().as_ref())?;
        for batch in batches {
            writer.write(batch)?;
        }
        writer.finish()?;
    }
    Ok(buffer.into_inner())
}

/// Attach or overwrite schema metadata on a `RecordBatch`.
///
/// Existing schema metadata is preserved unless a provided key overwrites it.
///
/// # Errors
///
/// Returns [`ArrowError`] when the batch cannot be rebuilt with the merged
/// schema metadata.
pub fn attach_record_batch_metadata<K, V, I>(
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

/// Attach or overwrite the canonical `trace_id` schema metadata entry.
///
/// # Errors
///
/// Returns [`ArrowError`] when the batch cannot be rebuilt with the updated
/// schema metadata.
pub fn attach_record_batch_trace_id(
    batch: &RecordBatch,
    trace_id: impl Into<String>,
) -> Result<RecordBatch, ArrowError> {
    attach_record_batch_metadata(batch, [(TRACE_ID_METADATA_KEY, trace_id.into())])
}

/// Decode Arrow IPC stream bytes into one or more `RecordBatch` values.
///
/// # Errors
///
/// Returns [`ArrowError`] when the payload is not valid Arrow IPC stream data.
pub fn decode_record_batches_ipc(payload: &[u8]) -> Result<Vec<RecordBatch>, ArrowError> {
    let cursor = Cursor::new(payload);
    let reader = StreamReader::try_new(cursor, None)?;
    reader.collect()
}

#[cfg(test)]
#[path = "../tests/unit/arrow_codec.rs"]
mod tests;
