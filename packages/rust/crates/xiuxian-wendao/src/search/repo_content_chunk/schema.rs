use std::sync::Arc;

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use xiuxian_vector_store::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
    VectorStoreError,
};

use crate::repo_index::RepoCodeDocument;

const CHUNK_SIZE: usize = 2_000;

const COLUMN_ID: &str = "id";
const COLUMN_PATH: &str = "path";
const COLUMN_PATH_FOLDED: &str = "path_folded";
const COLUMN_LANGUAGE: &str = "language";
const COLUMN_LINE_NUMBER: &str = "line_number";
const COLUMN_LINE_TEXT: &str = "line_text";
const COLUMN_LINE_TEXT_FOLDED: &str = "line_text_folded";
const COLUMN_SEARCH_TEXT: &str = "search_text";

#[derive(Debug, Clone)]
pub(crate) struct RepoContentChunkRow {
    pub(crate) path: String,
    pub(crate) path_folded: String,
    pub(crate) language: String,
    pub(crate) line_number: usize,
    pub(crate) line_text: String,
    pub(crate) line_text_folded: String,
}

pub(crate) fn repo_content_chunk_schema() -> Arc<LanceSchema> {
    Arc::new(LanceSchema::new(vec![
        LanceField::new(COLUMN_ID, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_PATH_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LANGUAGE, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LINE_NUMBER, LanceDataType::UInt64, false),
        LanceField::new(COLUMN_LINE_TEXT, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_LINE_TEXT_FOLDED, LanceDataType::Utf8, false),
        LanceField::new(COLUMN_SEARCH_TEXT, LanceDataType::Utf8, false),
    ]))
}

pub(crate) fn repo_content_chunk_engine_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new(COLUMN_ID, DataType::Utf8, false),
        Field::new(COLUMN_PATH, DataType::Utf8, false),
        Field::new(COLUMN_PATH_FOLDED, DataType::Utf8, false),
        Field::new(COLUMN_LANGUAGE, DataType::Utf8, false),
        Field::new(COLUMN_LINE_NUMBER, DataType::UInt64, false),
        Field::new(COLUMN_LINE_TEXT, DataType::Utf8, false),
        Field::new(COLUMN_LINE_TEXT_FOLDED, DataType::Utf8, false),
        Field::new(COLUMN_SEARCH_TEXT, DataType::Utf8, false),
    ]))
}

pub(super) fn rows_from_documents(documents: &[RepoCodeDocument]) -> Vec<RepoContentChunkRow> {
    let mut rows = Vec::new();
    for document in documents {
        let language = document.language.clone().unwrap_or_default();
        for (index, line) in document.contents.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            rows.push(RepoContentChunkRow {
                path: document.path.clone(),
                path_folded: document.path.to_ascii_lowercase(),
                language: language.clone(),
                line_number: index + 1,
                line_text: trimmed.to_string(),
                line_text_folded: trimmed.to_ascii_lowercase(),
            });
        }
    }
    rows
}

pub(super) fn repo_content_chunk_batches(
    rows: &[RepoContentChunkRow],
) -> Result<Vec<LanceRecordBatch>, VectorStoreError> {
    rows.chunks(CHUNK_SIZE)
        .map(batch_from_rows)
        .collect::<Result<Vec<_>, _>>()
}

fn batch_from_rows(rows: &[RepoContentChunkRow]) -> Result<LanceRecordBatch, VectorStoreError> {
    let schema = repo_content_chunk_schema();
    let ids = rows
        .iter()
        .map(|row| format!("{}:{}", row.path, row.line_number))
        .collect::<Vec<_>>();
    let paths = rows.iter().map(|row| row.path.clone()).collect::<Vec<_>>();
    let path_folded = rows
        .iter()
        .map(|row| row.path_folded.clone())
        .collect::<Vec<_>>();
    let languages = rows
        .iter()
        .map(|row| row.language.clone())
        .collect::<Vec<_>>();
    let line_numbers = rows
        .iter()
        .map(|row| u64::try_from(row.line_number).unwrap_or(u64::MAX))
        .collect::<Vec<_>>();
    let line_text = rows
        .iter()
        .map(|row| row.line_text.clone())
        .collect::<Vec<_>>();
    let line_text_folded = rows
        .iter()
        .map(|row| row.line_text_folded.clone())
        .collect::<Vec<_>>();
    let search_text = rows
        .iter()
        .map(|row| row.line_text.clone())
        .collect::<Vec<_>>();

    LanceRecordBatch::try_new(
        schema,
        vec![
            Arc::new(LanceStringArray::from(ids)),
            Arc::new(LanceStringArray::from(paths)),
            Arc::new(LanceStringArray::from(path_folded)),
            Arc::new(LanceStringArray::from(languages)),
            Arc::new(LanceUInt64Array::from(line_numbers)),
            Arc::new(LanceStringArray::from(line_text)),
            Arc::new(LanceStringArray::from(line_text_folded)),
            Arc::new(LanceStringArray::from(search_text)),
        ],
    )
    .map_err(VectorStoreError::Arrow)
}

pub(super) const fn projected_columns() -> [&'static str; 5] {
    [
        COLUMN_PATH,
        COLUMN_LANGUAGE,
        COLUMN_LINE_NUMBER,
        COLUMN_LINE_TEXT,
        COLUMN_LINE_TEXT_FOLDED,
    ]
}

pub(super) const fn query_projected_columns() -> [&'static str; 4] {
    [
        COLUMN_PATH,
        COLUMN_LANGUAGE,
        COLUMN_LINE_NUMBER,
        COLUMN_LINE_TEXT,
    ]
}

pub(super) const fn language_column() -> &'static str {
    COLUMN_LANGUAGE
}

pub(super) const fn path_column() -> &'static str {
    COLUMN_PATH
}

pub(super) const fn path_folded_column() -> &'static str {
    COLUMN_PATH_FOLDED
}

pub(super) const fn line_text_folded_column() -> &'static str {
    COLUMN_LINE_TEXT_FOLDED
}
