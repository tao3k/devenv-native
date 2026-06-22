//! Arrow IPC encoding helpers for record-batch payload transport.

#[cfg(not(feature = "vector-store"))]
use std::collections::HashMap;
use std::io::Cursor;
#[cfg(not(feature = "vector-store"))]
use std::sync::Arc;

#[cfg(not(feature = "vector-store"))]
use arrow::datatypes::Schema;
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use arrow_ipc::reader::StreamReader;
use arrow_ipc::writer::StreamWriter;

#[cfg(feature = "artifact-cache")]
use crate::artifact_cache::{
    ArtifactBlobCache, ArtifactBlobReadStatus, ArtifactBlobWrite, ArtifactBlobWriteOutcome,
    ArtifactKey,
};

#[cfg(not(feature = "vector-store"))]
const TRACE_ID_METADATA_KEY: &str = "trace_id";

/// Encode a single Arrow `RecordBatch` into IPC stream bytes.
///
/// # Errors
///
/// Returns [`ArrowError`] when Arrow IPC stream construction fails.
#[cfg(not(feature = "vector-store"))]
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
#[cfg(not(feature = "vector-store"))]
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
#[cfg(not(feature = "vector-store"))]
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

/// Encode and write Arrow IPC batch bytes to an artifact cache key.
///
/// # Errors
///
/// Returns [`ArrowError`] when IPC encoding fails or the artifact backend
/// cannot write the payload.
#[cfg(feature = "artifact-cache")]
pub fn write_record_batches_ipc_artifact(
    cache: &dyn ArtifactBlobCache,
    key: &ArtifactKey,
    batches: &[RecordBatch],
) -> Result<ArtifactBlobWriteOutcome, ArrowError> {
    let bytes = encode_record_batches_ipc(batches)?;
    cache
        .write(key, ArtifactBlobWrite::new(bytes.as_slice()))
        .map_err(artifact_cache_arrow_error)
}

/// Read and decode Arrow IPC batch bytes from an artifact cache key.
///
/// A backend pressure result is treated as a non-hit so callers can rebuild the
/// projection through the same precision path as a normal miss.
///
/// # Errors
///
/// Returns [`ArrowError`] when the artifact backend read fails or cached bytes
/// are not valid Arrow IPC stream data.
#[cfg(feature = "artifact-cache")]
pub fn read_record_batches_ipc_artifact(
    cache: &dyn ArtifactBlobCache,
    key: &ArtifactKey,
) -> Result<Option<Vec<RecordBatch>>, ArrowError> {
    match cache
        .read_with_status(key)
        .map_err(artifact_cache_arrow_error)?
    {
        ArtifactBlobReadStatus::Hit(read) => decode_record_batches_ipc(read.bytes()).map(Some),
        ArtifactBlobReadStatus::Miss | ArtifactBlobReadStatus::Throttled => Ok(None),
    }
}

#[cfg(feature = "artifact-cache")]
fn artifact_cache_arrow_error(error: crate::artifact_cache::ArtifactCacheError) -> ArrowError {
    ArrowError::ExternalError(Box::new(error))
}
