use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
    VectorStoreError,
};

use crate::gateway::studio::types::ReferenceSearchHit;

const CHUNK_SIZE: usize = 1_000;

const COLUMN_ID: &str = "id";
const COLUMN_NAME: &str = "name";
const COLUMN_NAME_FOLDED: &str = "name_folded";
const COLUMN_PATH: &str = "path";
const COLUMN_LINE: &str = "line";
const COLUMN_COLUMN: &str = "column";
const COLUMN_LINE_TEXT: &str = "line_text";
const COLUMN_HIT_JSON: &str = "hit_json";

pub(crate) fn reference_occurrence_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LINE, LanceDataType::UInt64, false),
        LanceField::new(COLUMN_COLUMN, LanceDataType::UInt64, false),
        LanceField::new(COLUMN_LINE_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_HIT_JSON, LanceDataType::Utf8, false),
    ]))
}

pub(crate) fn reference_occurrence_batches(
    hits: &[ReferenceSearchHit],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    hits.chunks(CHUNK_SIZE)
        .map(batch_from_hits)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_hits(hits: &[ReferenceSearchHit]) -> Result<LanceRecordBatch, VectorStoreError> {
    let schema = reference_occurrence_schema();
    let ids = hits
        .iter()
        .map(|hit| format!("{}:{}:{}:{}", hit.path, hit.line, hit.column, hit.name))
        .collect::<Vec<_>>();
    let names = hits.iter().map(|hit| hit.name.clone()).collect::<Vec<_>>();
    let name_folded = hits
        .iter()
        .map(|hit| hit.name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>();
    let lines = hits
        .iter()
        .map(|hit| u64::try_from(hit.line).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    let columns = hits
        .iter()
        .map(|hit| u64::try_from(hit.column).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    let line_text = hits
        .iter()
        .map(|hit| hit.line_text.clone())
        .collect::<Vec<_>>();
    let hit_json = hits
        .iter()
        .map(|hit| {
            serde_json::to_string(hit).map_err(|error| {
                VectorStoreError::General(format!("serialize reference hit: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(name_folded)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceUInt64Array::from(lines)),
            Arc::new(LanceUInt64Array::from(columns)),
            Arc::new(LanceStringArray::from(line_text)),
            Arc::new(LanceStringArray::from(hit_json)),
        ],
    )
    .map_err(VectorStoreError::Arrow)
}

pub(super) const fn projected_columns() -> [&'static str; 5] {
    [
        COLUMN_ID,
        COLUMN_PATH,
        COLUMN_LINE,
        COLUMN_COLUMN,
        COLUMN_LINE_TEXT,
    ]
}

pub(super) const fn filter_column() -> &'static str {
    COLUMN_NAME_FOLDED
}

pub(super) const fn path_column() -> &'static str {
    COLUMN_PATH
}

pub(super) const fn id_column() -> &'static str {
    COLUMN_ID
}

pub(super) const fn hit_json_column() -> &'static str {
    COLUMN_HIT_JSON
}
