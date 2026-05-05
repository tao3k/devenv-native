//! Arrow IPC stream helpers for the link-graph snapshot cache.

use std::io::Cursor;

use arrow::array::{Array, Int64Array, ListArray, StringArray};
use arrow::ipc::reader::StreamReader;
use arrow::ipc::writer::StreamWriter;
use arrow::record_batch::RecordBatch;

pub(super) fn encode_batch(batch: &RecordBatch) -> Result<Vec<u8>, String> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut writer = StreamWriter::try_new(&mut buffer, batch.schema().as_ref())
            .map_err(|error| format!("open link-graph Arrow IPC writer: {error}"))?;
        writer
            .write(batch)
            .map_err(|error| format!("write link-graph Arrow IPC batch: {error}"))?;
        writer
            .finish()
            .map_err(|error| format!("finish link-graph Arrow IPC stream: {error}"))?;
    }
    Ok(buffer.into_inner())
}

pub(super) fn decode_single_batch(
    payload: &[u8],
    stream_name: &str,
) -> Result<RecordBatch, String> {
    let reader = StreamReader::try_new(Cursor::new(payload), None)
        .map_err(|error| format!("open link-graph Arrow {stream_name} stream: {error}"))?;
    let batches = reader
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("decode link-graph Arrow {stream_name} stream: {error}"))?;
    let [batch] = batches.as_slice() else {
        return Err(format!(
            "expected one link-graph Arrow {stream_name} batch, got {}",
            batches.len()
        ));
    };
    Ok(batch.clone())
}

pub(super) fn required_column<'a, T: Array + 'static>(
    batch: &'a RecordBatch,
    column_name: &str,
) -> Result<&'a T, String> {
    batch
        .column_by_name(column_name)
        .and_then(|column| column.as_any().downcast_ref::<T>())
        .ok_or_else(|| format!("missing link-graph Arrow column `{column_name}`"))
}

pub(super) fn string_at<'a>(
    array: &'a StringArray,
    row: usize,
    column_name: &str,
) -> Result<&'a str, String> {
    if array.is_null(row) {
        return Err(format!(
            "unexpected null in link-graph Arrow column `{column_name}`"
        ));
    }
    Ok(array.value(row))
}

pub(super) fn optional_string_at(array: &StringArray, row: usize) -> Option<String> {
    if array.is_null(row) {
        None
    } else {
        Some(array.value(row).to_string())
    }
}

pub(super) fn optional_i64_at(array: &Int64Array, row: usize) -> Option<i64> {
    if array.is_null(row) {
        None
    } else {
        Some(array.value(row))
    }
}

pub(super) fn string_list_at(
    array: &ListArray,
    row: usize,
    column_name: &str,
) -> Result<Vec<String>, String> {
    if array.is_null(row) {
        return Ok(Vec::new());
    }
    let values = array.value(row);
    let Some(strings) = values.as_any().downcast_ref::<StringArray>() else {
        return Err(format!(
            "expected Utf8 values in link-graph Arrow list column `{column_name}`"
        ));
    };
    Ok((0..strings.len())
        .map(|index| {
            if strings.is_null(index) {
                String::new()
            } else {
                strings.value(index).to_string()
            }
        })
        .collect())
}
