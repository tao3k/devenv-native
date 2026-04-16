use arrow::array::{StringArray, StringViewArray, UInt64Array};
use xiuxian_vector_store::EngineRecordBatch;

use crate::search::reference_occurrence::ReferenceOccurrenceSearchError;

pub(super) fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(super) fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub(super) fn string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, ReferenceOccurrenceSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        ReferenceOccurrenceSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;
    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }
    Err(ReferenceOccurrenceSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

pub(super) fn u64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a UInt64Array, ReferenceOccurrenceSearchError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            ReferenceOccurrenceSearchError::Decode(format!("missing engine u64 column `{name}`"))
        })
}

#[derive(Clone, Copy)]
pub(super) enum EngineStringColumn<'a> {
    Utf8(&'a StringArray),
    Utf8View(&'a StringViewArray),
}

impl<'a> EngineStringColumn<'a> {
    pub(super) fn value(self, row: usize) -> &'a str {
        match self {
            Self::Utf8(column) => column.value(row),
            Self::Utf8View(column) => column.value(row),
        }
    }
}
