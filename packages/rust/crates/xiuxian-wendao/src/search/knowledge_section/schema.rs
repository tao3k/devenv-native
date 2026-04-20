use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, VectorStoreError,
};

#[derive(Debug, Clone)]
pub(super) struct KnowledgeSectionRow {
    pub id: String,
    pub path: String,
    pub stem: String,
    pub title: Option<String>,
    pub best_section: Option<String>,
    pub search_text: String,
    pub hit_json: String,
}

const CHUNK_SIZE: usize = 1_000;

const COLUMN_ID: &str = "id";
const COLUMN_PATH: &str = "path";
const COLUMN_STEM: &str = "stem";
const COLUMN_TITLE: &str = "title";
const COLUMN_BEST_SECTION: &str = "best_section";
const COLUMN_SEARCH_TEXT: &str = "search_text";
const COLUMN_SEARCH_TEXT_FOLDED: &str = "search_text_folded";
const COLUMN_HIT_JSON: &str = "hit_json";

pub(super) fn knowledge_section_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_STEM, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_TITLE, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_BEST_SECTION, LanceDataType::Utf8, true),
        LanceField::new(COLUMN_SEARCH_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SEARCH_TEXT_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_HIT_JSON, LanceDataType::Utf8, false),
    ]))
}

pub(super) fn knowledge_section_batches(
    rows: &[KnowledgeSectionRow],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    rows.chunks(CHUNK_SIZE)
        .map(batch_from_hits)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_hits(rows: &[KnowledgeSectionRow]) -> Result<LanceRecordBatch, VectorStoreError> {
    let schema = knowledge_section_schema();
    let ids = rows.iter().map(|row| row.id.clone()).collect::<Vec<_>>();
    let paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let stems = rows.iter().map(|row| row.stem.clone()).collect::<Vec<_>>();
    let titles = rows.iter().map(|row| row.title.clone()).collect::<Vec<_>>();
    let best_sections = rows
        .iter()
        .map(|row| row.best_section.clone())
        .collect::<Vec<_>>();
    let search_text = rows
        .iter()
        .map(|row| row.search_text.clone())
        .collect::<Vec<_>>();
    let search_text_folded = search_text
        .iter()
        .map(|value| value.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let hit_json = rows
        .iter()
        .map(|row| row.hit_json.clone())
        .collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceStringArray::from(stems)),
            Arc::new(LanceStringArray::from(titles)),
            Arc::new(LanceStringArray::from(best_sections)),
            Arc::new(LanceStringArray::from(search_text)),
            Arc::new(LanceStringArray::from(search_text_folded)),
            Arc::new(LanceStringArray::from(hit_json)),
        ],
    )
    .map_err(VectorStoreError::Arrow)
}

pub(super) const fn projected_columns() -> [&'static str; 6] {
    [
        COLUMN_ID,
        COLUMN_PATH,
        COLUMN_STEM,
        COLUMN_TITLE,
        COLUMN_BEST_SECTION,
        COLUMN_SEARCH_TEXT_FOLDED,
    ]
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
