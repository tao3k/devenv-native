//! Arrow IPC conversion between engine-native and Lance-native record batches.

use std::io::Cursor;

use arrow::record_batch::RecordBatch as EngineRecordBatch;
use arrow_ipc::reader::StreamReader as EngineStreamReader;
use arrow_ipc::writer::StreamWriter as EngineStreamWriter;
use lance_arrow_ipc::reader::StreamReader as LanceStreamReader;
use lance_arrow_ipc::writer::StreamWriter as LanceStreamWriter;

use crate::{LanceRecordBatch, VectorStoreError};

fn decode_engine_batches(payload: &[u8]) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    let reader = EngineStreamReader::try_new(Cursor::new(payload), None)?;
    reader.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn decode_lance_batches(payload: &[u8]) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    let reader = LanceStreamReader::try_new(Cursor::new(payload), None)?;
    reader.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn encode_engine_batches(batches: &[EngineRecordBatch]) -> Result<Vec<u8>, VectorStoreError> {
    let Some(first_batch) = batches.first() else {
        return Ok(Vec::new());
    };
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = EngineStreamWriter::try_new(&mut buffer, first_batch.schema().as_ref())?;
        for batch in batches {
            writer.write(batch)?;
        }
        writer.finish()?;
    }
    Ok(buffer.into_inner())
}

fn encode_lance_batches(batches: &[LanceRecordBatch]) -> Result<Vec<u8>, VectorStoreError> {
    let Some(first_batch) = batches.first() else {
        return Ok(Vec::new());
    };
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = LanceStreamWriter::try_new(&mut buffer, first_batch.schema().as_ref())?;
        for batch in batches {
            writer.write(batch)?;
        }
        writer.finish()?;
    }
    Ok(buffer.into_inner())
}

/// Convert a Lance-owned record batch into a workspace Arrow/DataFusion record batch.
///
/// # Errors
///
/// Returns an error when Arrow IPC encoding or decoding fails.
pub fn lance_batch_to_engine_batch(
    batch: &LanceRecordBatch,
) -> Result<EngineRecordBatch, VectorStoreError> {
    let mut batches = lance_batches_to_engine_batches(std::slice::from_ref(batch))?;
    batches.pop().ok_or_else(|| {
        VectorStoreError::General(
            "IPC conversion from Lance batch to engine batch produced no rows".to_string(),
        )
    })
}

/// Convert multiple Lance-owned batches into workspace Arrow/DataFusion batches.
///
/// # Errors
///
/// Returns an error when Arrow IPC encoding or decoding fails.
pub fn lance_batches_to_engine_batches(
    batches: &[LanceRecordBatch],
) -> Result<Vec<EngineRecordBatch>, VectorStoreError> {
    if batches.is_empty() {
        return Ok(Vec::new());
    }
    decode_engine_batches(&encode_lance_batches(batches)?)
}

/// Convert a workspace Arrow/DataFusion batch into a Lance-owned record batch.
///
/// # Errors
///
/// Returns an error when Arrow IPC encoding or decoding fails.
pub fn engine_batch_to_lance_batch(
    batch: &EngineRecordBatch,
) -> Result<LanceRecordBatch, VectorStoreError> {
    let mut batches = engine_batches_to_lance_batches(std::slice::from_ref(batch))?;
    batches.pop().ok_or_else(|| {
        VectorStoreError::General(
            "IPC conversion from engine batch to Lance batch produced no rows".to_string(),
        )
    })
}

/// Convert multiple workspace Arrow/DataFusion batches into Lance-owned batches.
///
/// # Errors
///
/// Returns an error when Arrow IPC encoding or decoding fails.
pub fn engine_batches_to_lance_batches(
    batches: &[EngineRecordBatch],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    if batches.is_empty() {
        return Ok(Vec::new());
    }
    decode_lance_batches(&encode_engine_batches(batches)?)
}
