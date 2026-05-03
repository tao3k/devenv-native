use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, VectorStoreError,
};

use crate::search::contracts::AttachmentSearchHit;

const CHUNK_SIZE: usize = 1_000;

const COLUMN_ID: &str = "id";
const COLUMN_SOURCE_PATH: &str = "source_path";
const COLUMN_SOURCE_TITLE: &str = "source_title";
const COLUMN_SOURCE_STEM: &str = "source_stem";
const COLUMN_ATTACHMENT_PATH: &str = "attachment_path";
const COLUMN_ATTACHMENT_NAME: &str = "attachment_name";
const COLUMN_ATTACHMENT_EXT: &str = "attachment_ext";
const COLUMN_KIND: &str = "kind";
const COLUMN_SEARCH_TEXT: &str = "search_text";
const COLUMN_SOURCE_PATH_FOLDED: &str = "source_path_folded";
const COLUMN_SOURCE_TITLE_FOLDED: &str = "source_title_folded";
const COLUMN_SOURCE_STEM_FOLDED: &str = "source_stem_folded";
const COLUMN_ATTACHMENT_PATH_FOLDED: &str = "attachment_path_folded";
const COLUMN_ATTACHMENT_NAME_FOLDED: &str = "attachment_name_folded";
const COLUMN_HIT_JSON: &str = "hit_json";

pub(super) fn attachment_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_TITLE, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_STEM, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ATTACHMENT_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ATTACHMENT_NAME, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ATTACHMENT_EXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_KIND, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SEARCH_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_PATH_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_TITLE_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SOURCE_STEM_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ATTACHMENT_PATH_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_ATTACHMENT_NAME_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_HIT_JSON, LanceDataType::Utf8, false),
    ]))
}

pub(super) fn attachment_batches(
    hits: &[AttachmentSearchHit],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    hits.chunks(CHUNK_SIZE)
        .map(batch_from_hits)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_hits(hits: &[AttachmentSearchHit]) -> Result<LanceRecordBatch, VectorStoreError> {
    let schema = attachment_schema();
    let ids = hits
        .iter()
        .map(|hit| hit.attachment_id.clone())
        .collect::<Vec<_>>();
    let source_path = hits
        .iter()
        .map(|hit| hit.source_path.clone())
        .collect::<Vec<_>>();
    let source_title = hits
        .iter()
        .map(|hit| hit.source_title.clone())
        .collect::<Vec<_>>();
    let source_stem = hits
        .iter()
        .map(|hit| hit.source_stem.clone())
        .collect::<Vec<_>>();
    let attachment_path = hits
        .iter()
        .map(|hit| hit.attachment_path.clone())
        .collect::<Vec<_>>();
    let attachment_name = hits
        .iter()
        .map(|hit| hit.attachment_name.clone())
        .collect::<Vec<_>>();
    let attachment_ext = hits
        .iter()
        .map(|hit| hit.attachment_ext.clone())
        .collect::<Vec<_>>();
    let kind = hits.iter().map(|hit| hit.kind.clone()).collect::<Vec<_>>();
    let search_text = hits
        .iter()
        .map(|hit| {
            [
                hit.attachment_path.as_str(),
                hit.attachment_name.as_str(),
                hit.source_path.as_str(),
                hit.source_title.as_str(),
                hit.source_stem.as_str(),
            ]
            .join(" ")
        })
        .collect::<Vec<_>>();
    let source_path_folded = hits
        .iter()
        .map(|hit| hit.source_path.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let source_title_folded = hits
        .iter()
        .map(|hit| hit.source_title.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let source_stem_folded = hits
        .iter()
        .map(|hit| hit.source_stem.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let attachment_path_folded = hits
        .iter()
        .map(|hit| hit.attachment_path.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let attachment_name_folded = hits
        .iter()
        .map(|hit| hit.attachment_name.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let hit_json = hits
        .iter()
        .map(|hit| {
            serde_json::to_string(hit).map_err(|error| {
                VectorStoreError::General(format!("serialize attachment hit: {error}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(source_path)),
            Arc::new(LanceStringArray::from(source_title)),
            Arc::new(LanceStringArray::from(source_stem)),
            Arc::new(LanceStringArray::from(attachment_path)),
            Arc::new(LanceStringArray::from(attachment_name)),
            Arc::new(LanceStringArray::from(attachment_ext)),
            Arc::new(LanceStringArray::from(kind)),
            Arc::new(LanceStringArray::from(search_text)),
            Arc::new(LanceStringArray::from(source_path_folded)),
            Arc::new(LanceStringArray::from(source_title_folded)),
            Arc::new(LanceStringArray::from(source_stem_folded)),
            Arc::new(LanceStringArray::from(attachment_path_folded)),
            Arc::new(LanceStringArray::from(attachment_name_folded)),
            Arc::new(LanceStringArray::from(hit_json)),
        ],
    )
    .map_err(VectorStoreError::Arrow)
}

pub(super) const fn projected_columns() -> [&'static str; 13] {
    [
        COLUMN_ID,
        COLUMN_SOURCE_PATH,
        COLUMN_SOURCE_TITLE,
        COLUMN_SOURCE_STEM,
        COLUMN_ATTACHMENT_PATH,
        COLUMN_ATTACHMENT_NAME,
        COLUMN_ATTACHMENT_EXT,
        COLUMN_KIND,
        COLUMN_SOURCE_PATH_FOLDED,
        COLUMN_SOURCE_TITLE_FOLDED,
        COLUMN_SOURCE_STEM_FOLDED,
        COLUMN_ATTACHMENT_PATH_FOLDED,
        COLUMN_ATTACHMENT_NAME_FOLDED,
    ]
}

pub(super) const fn projected_columns_with_hit_json() -> [&'static str; 14] {
    [
        COLUMN_ID,
        COLUMN_SOURCE_PATH,
        COLUMN_SOURCE_TITLE,
        COLUMN_SOURCE_STEM,
        COLUMN_ATTACHMENT_PATH,
        COLUMN_ATTACHMENT_NAME,
        COLUMN_ATTACHMENT_EXT,
        COLUMN_KIND,
        COLUMN_SOURCE_PATH_FOLDED,
        COLUMN_SOURCE_TITLE_FOLDED,
        COLUMN_SOURCE_STEM_FOLDED,
        COLUMN_ATTACHMENT_PATH_FOLDED,
        COLUMN_ATTACHMENT_NAME_FOLDED,
        COLUMN_HIT_JSON,
    ]
}

pub(super) const fn attachment_ext_column() -> &'static str {
    COLUMN_ATTACHMENT_EXT
}

pub(super) const fn id_column() -> &'static str {
    COLUMN_ID
}

pub(super) const fn kind_column() -> &'static str {
    COLUMN_KIND
}

pub(super) const fn hit_json_column() -> &'static str {
    COLUMN_HIT_JSON
}

pub(super) const fn attachment_name_column() -> &'static str {
    COLUMN_ATTACHMENT_NAME
}

pub(super) const fn attachment_name_folded_column() -> &'static str {
    COLUMN_ATTACHMENT_NAME_FOLDED
}

pub(super) const fn source_path_column() -> &'static str {
    COLUMN_SOURCE_PATH
}
