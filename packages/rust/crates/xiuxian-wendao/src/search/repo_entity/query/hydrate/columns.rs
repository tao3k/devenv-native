use arrow::array::{Array, Float64Array, ListArray, StringArray, StringViewArray, UInt32Array};
use xiuxian_db_store::EngineRecordBatch;

use crate::search::repo_entity::query::lookup::RepoEntitySearchError;
use crate::search::repo_entity::schema::{
    COLUMN_ATTRIBUTES_JSON, COLUMN_AUDIT_STATUS, COLUMN_HIERARCHICAL_URI, COLUMN_HIERARCHY,
    COLUMN_IMPLICIT_BACKLINK_ITEMS_JSON, COLUMN_IMPLICIT_BACKLINKS, COLUMN_LINE_END,
    COLUMN_LINE_START, COLUMN_MODULE_ID, COLUMN_NAME, COLUMN_PATH, COLUMN_PROJECTION_PAGE_IDS,
    COLUMN_QUALIFIED_NAME, COLUMN_SALIENCY_SCORE, COLUMN_SIGNATURE, COLUMN_SUMMARY,
    COLUMN_SYMBOL_KIND, COLUMN_VERIFICATION_STATE, hit_json_column, id_column,
};

#[derive(Clone, Copy)]
pub(crate) enum EngineStringColumn<'a> {
    Utf8(&'a StringArray),
    Utf8View(&'a StringViewArray),
}

impl<'a> EngineStringColumn<'a> {
    pub(crate) fn value(self, row: usize) -> &'a str {
        match self {
            Self::Utf8(column) => column.value(row),
            Self::Utf8View(column) => column.value(row),
        }
    }

    pub(crate) fn is_null(self, row: usize) -> bool {
        match self {
            Self::Utf8(column) => column.is_null(row),
            Self::Utf8View(column) => column.is_null(row),
        }
    }
}

pub(crate) fn typed_repo_entity_columns() -> Vec<String> {
    [
        id_column(),
        COLUMN_NAME,
        COLUMN_QUALIFIED_NAME,
        COLUMN_PATH,
        COLUMN_SYMBOL_KIND,
        COLUMN_MODULE_ID,
        COLUMN_SIGNATURE,
        COLUMN_SUMMARY,
        COLUMN_LINE_START,
        COLUMN_LINE_END,
        COLUMN_AUDIT_STATUS,
        COLUMN_VERIFICATION_STATE,
        COLUMN_ATTRIBUTES_JSON,
        COLUMN_HIERARCHICAL_URI,
        COLUMN_HIERARCHY,
        COLUMN_IMPLICIT_BACKLINKS,
        COLUMN_IMPLICIT_BACKLINK_ITEMS_JSON,
        COLUMN_PROJECTION_PAGE_IDS,
        COLUMN_SALIENCY_SCORE,
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

pub(crate) fn id_filter_expression(ids: &[String]) -> String {
    let escaped = ids
        .iter()
        .map(|value| format!("'{}'", value.replace('\'', "''")))
        .collect::<Vec<_>>();
    format!("{} IN ({})", id_column(), escaped.join(","))
}

pub(crate) fn engine_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, RepoEntitySearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        RepoEntitySearchError::Decode(format!("missing engine string column `{name}`"))
    })?;

    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }

    Err(RepoEntitySearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

pub(crate) fn engine_float64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a Float64Array, RepoEntitySearchError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| RepoEntitySearchError::Decode(format!("missing engine f64 column `{name}`")))
}

pub(crate) fn engine_uint32_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a UInt32Array, RepoEntitySearchError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<UInt32Array>())
        .ok_or_else(|| RepoEntitySearchError::Decode(format!("missing engine u32 column `{name}`")))
}

pub(crate) fn engine_list_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a ListArray, RepoEntitySearchError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<ListArray>())
        .ok_or_else(|| {
            RepoEntitySearchError::Decode(format!("missing engine list column `{name}`"))
        })
}

pub(crate) fn optional_engine_string_value(
    column: EngineStringColumn<'_>,
    row: usize,
) -> Option<String> {
    if column.is_null(row) {
        return None;
    }

    let value = column.value(row).trim().to_string();
    (!value.is_empty()).then_some(value)
}

pub(crate) fn optional_engine_u32_value(column: &UInt32Array, row: usize) -> Option<u32> {
    (!column.is_null(row)).then(|| column.value(row))
}

pub(crate) fn engine_list_string_values(column: &ListArray, row: usize) -> Vec<String> {
    if column.is_null(row) {
        return Vec::new();
    }

    let values = column.value(row);
    string_values_from_array(values.as_ref())
}

pub(crate) fn hit_json_projection_columns() -> Vec<String> {
    vec![id_column().to_string(), hit_json_column().to_string()]
}

fn string_values_from_array(values: &dyn Array) -> Vec<String> {
    if let Some(strings) = values.as_any().downcast_ref::<StringArray>() {
        return (0..strings.len())
            .filter(|index| !strings.is_null(*index))
            .map(|index| strings.value(index).trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
    }

    if let Some(strings) = values.as_any().downcast_ref::<StringViewArray>() {
        return (0..strings.len())
            .filter(|index| !strings.is_null(*index))
            .map(|index| strings.value(index).trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
    }

    Vec::new()
}
