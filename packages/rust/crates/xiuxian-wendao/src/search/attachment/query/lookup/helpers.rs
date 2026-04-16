use arrow::array::{StringArray, StringViewArray};
use xiuxian_vector_store::EngineRecordBatch;

use super::types::AttachmentSearchError;

pub(super) fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(super) fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

pub(super) fn string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, AttachmentSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        AttachmentSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;
    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }
    Err(AttachmentSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

pub(super) enum EngineStringColumn<'a> {
    Utf8(&'a StringArray),
    Utf8View(&'a StringViewArray),
}

impl EngineStringColumn<'_> {
    pub(super) fn value(&self, row: usize) -> &str {
        match self {
            Self::Utf8(column) => column.value(row),
            Self::Utf8View(column) => column.value(row),
        }
    }
}
