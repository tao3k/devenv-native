use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
    VectorStoreError,
};

use crate::gateway::studio::types::AstSearchHit;

const CHUNK_SIZE: usize = 1_000;

const COLUMN_ID: &str = "id";
const COLUMN_NAME: &str = "name";
const COLUMN_NAME_FOLDED: &str = "name_folded";
const COLUMN_LANGUAGE: &str = "language";
const COLUMN_NODE_KIND: &str = "node_kind";
const COLUMN_SIGNATURE: &str = "signature";
const COLUMN_OWNER_TITLE: &str = "owner_title";
const COLUMN_PATH: &str = "path";
const COLUMN_LINE_START: &str = "line_start";
const COLUMN_HIT_JSON: &str = "hit_json";
const COLUMN_SEARCH_TEXT: &str = "search_text";

pub(super) fn local_symbol_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NAME_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LANGUAGE, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_NODE_KIND, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_SIGNATURE, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_OWNER_TITLE, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LINE_START, LanceDataType::UInt64, false),
        LanceField::new(COLUMN_SEARCH_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_HIT_JSON, LanceDataType::Utf8, false),
    ]))
}

pub(super) fn local_symbol_batches(
    hits: &[AstSearchHit],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    hits.chunks(CHUNK_SIZE)
        .map(batch_from_hits)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_hits(hits: &[AstSearchHit]) -> Result<LanceRecordBatch, VectorStoreError> {
    let schema = local_symbol_schema();
    let ids = hits
        .iter()
        .map(|hit| {
            format!(
                "{}:{}:{}:{}",
                hit.path, hit.line_start, hit.line_end, hit.name
            )
        })
        .collect::<Vec<_>>();
    let names = hits.iter().map(|hit| hit.name.clone()).collect::<Vec<_>>();
    let name_folded = hits
        .iter()
        .map(|hit| hit.name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let languages = hits
        .iter()
        .map(|hit| hit.language.clone())
        .collect::<Vec<_>>();
    let node_kinds = hits
        .iter()
        .map(|hit| hit.node_kind.clone())
        .collect::<Vec<_>>();
    let signatures = hits
        .iter()
        .map(|hit| hit.signature.clone())
        .collect::<Vec<_>>();
    let owner_titles = hits
        .iter()
        .map(|hit| hit.owner_title.clone())
        .collect::<Vec<_>>();
    let paths = hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>();
    let line_starts = hits
        .iter()
        .map(|hit| u64::try_from(hit.line_start).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    let search_text = hits
        .iter()
        .map(|hit| {
            [
                hit.name.as_str(),
                hit.signature.as_str(),
                hit.owner_title.as_deref().unwrap_or(""),
            ]
            .join(" ")
        })
        .collect::<Vec<_>>();
    let hit_json = hits
        .iter()
        .map(|hit| {
            serde_json::to_string(hit)
                .map_err(|error| VectorStoreError::General(format!("serialize ast hit: {error}")))
        })
        .collect::<Result<Vec<_>, _>>()?;

    LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(names)),
            Arc::new(LanceStringArray::from(name_folded)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceStringArray::from(node_kinds)),
            Arc::new(LanceStringArray::from(signatures)),
            Arc::new(LanceStringArray::from(owner_titles)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceUInt64Array::from(line_starts)),
            Arc::new(LanceStringArray::from(search_text)),
            Arc::new(LanceStringArray::from(hit_json)),
        ],
    )
    .map_err(VectorStoreError::Arrow)
}

pub(super) const fn projected_columns() -> [&'static str; 7] {
    [
        COLUMN_ID,
        COLUMN_NAME,
        COLUMN_NAME_FOLDED,
        COLUMN_SIGNATURE,
        COLUMN_OWNER_TITLE,
        COLUMN_PATH,
        COLUMN_LINE_START,
    ]
}

pub(super) const fn suggestion_columns() -> [&'static str; 4] {
    [
        COLUMN_NAME,
        COLUMN_NAME_FOLDED,
        COLUMN_LANGUAGE,
        COLUMN_NODE_KIND,
    ]
}

pub(super) const fn hit_json_column() -> &'static str {
    COLUMN_HIT_JSON
}

pub(super) const fn id_column() -> &'static str {
    COLUMN_ID
}

pub(super) const fn path_column() -> &'static str {
    COLUMN_PATH
}
