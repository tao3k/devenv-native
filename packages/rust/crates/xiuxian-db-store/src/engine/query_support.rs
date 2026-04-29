//! Arrow-native retrieval batch helpers shared by Wendao query-core adapters.

use std::collections::BTreeSet;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Float64Array, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

use crate::VectorStoreError;

/// Stable candidate identifier column.
pub const RETRIEVAL_ID_COLUMN: &str = "id";
/// Repository-relative path column.
pub const RETRIEVAL_PATH_COLUMN: &str = "path";
/// Repository identifier column.
pub const RETRIEVAL_REPO_COLUMN: &str = "repo";
/// Display title column.
pub const RETRIEVAL_TITLE_COLUMN: &str = "title";
/// Retrieval score column.
pub const RETRIEVAL_SCORE_COLUMN: &str = "score";
/// Backend source label column.
pub const RETRIEVAL_SOURCE_COLUMN: &str = "source";
/// Optional snippet column.
pub const RETRIEVAL_SNIPPET_COLUMN: &str = "snippet";
/// Optional doc-type column.
pub const RETRIEVAL_DOC_TYPE_COLUMN: &str = "doc_type";
/// Optional match-reason column.
pub const RETRIEVAL_MATCH_REASON_COLUMN: &str = "match_reason";
/// Optional best-section column.
pub const RETRIEVAL_BEST_SECTION_COLUMN: &str = "best_section";
/// Optional language column.
pub const RETRIEVAL_LANGUAGE_COLUMN: &str = "language";
/// Optional line-number column.
pub const RETRIEVAL_LINE_COLUMN: &str = "line";

/// Arrow-friendly row model used by retrieval adapters.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RetrievalRow {
    /// Stable candidate identifier.
    pub id: String,
    /// Repository-relative path.
    pub path: String,
    /// Optional repository identifier.
    pub repo: Option<String>,
    /// Optional display title.
    pub title: Option<String>,
    /// Optional normalized score.
    pub score: Option<f64>,
    /// Adapter/backend source label.
    pub source: String,
    /// Optional preview snippet.
    pub snippet: Option<String>,
    /// Optional doc type.
    pub doc_type: Option<String>,
    /// Optional match reason.
    pub match_reason: Option<String>,
    /// Optional best section.
    pub best_section: Option<String>,
    /// Optional language label.
    pub language: Option<String>,
    /// Optional 1-based line number.
    pub line: Option<u64>,
}

/// Return the canonical Arrow schema for retrieval candidate batches.
#[must_use]
pub fn retrieval_result_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new(RETRIEVAL_ID_COLUMN, DataType::Utf8, false),
        Field::new(RETRIEVAL_PATH_COLUMN, DataType::Utf8, false),
        Field::new(RETRIEVAL_REPO_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_TITLE_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_SCORE_COLUMN, DataType::Float64, true),
        Field::new(RETRIEVAL_SOURCE_COLUMN, DataType::Utf8, false),
        Field::new(RETRIEVAL_SNIPPET_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_DOC_TYPE_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_MATCH_REASON_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_BEST_SECTION_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_LANGUAGE_COLUMN, DataType::Utf8, true),
        Field::new(RETRIEVAL_LINE_COLUMN, DataType::UInt64, true),
    ]))
}

/// Return the canonical retrieval payload column order.
#[must_use]
pub fn retrieval_result_columns() -> Vec<String> {
    vec![
        RETRIEVAL_ID_COLUMN.to_string(),
        RETRIEVAL_PATH_COLUMN.to_string(),
        RETRIEVAL_REPO_COLUMN.to_string(),
        RETRIEVAL_TITLE_COLUMN.to_string(),
        RETRIEVAL_SCORE_COLUMN.to_string(),
        RETRIEVAL_SOURCE_COLUMN.to_string(),
        RETRIEVAL_SNIPPET_COLUMN.to_string(),
        RETRIEVAL_DOC_TYPE_COLUMN.to_string(),
        RETRIEVAL_MATCH_REASON_COLUMN.to_string(),
        RETRIEVAL_BEST_SECTION_COLUMN.to_string(),
        RETRIEVAL_LANGUAGE_COLUMN.to_string(),
        RETRIEVAL_LINE_COLUMN.to_string(),
    ]
}

/// Convert retrieval rows into a canonical Arrow record batch.
///
/// # Errors
///
/// Returns an error when the canonical retrieval batch cannot be materialized.
pub fn retrieval_rows_to_record_batch(
    rows: &[RetrievalRow],
) -> Result<RecordBatch, VectorStoreError> {
    let schema = retrieval_result_schema();
    let ids = StringArray::from(
        rows.iter()
            .map(|row| Some(row.id.as_str()))
            .collect::<Vec<_>>(),
    );
    let paths = StringArray::from(
        rows.iter()
            .map(|row| Some(row.path.as_str()))
            .collect::<Vec<_>>(),
    );
    let repos = StringArray::from(
        rows.iter()
            .map(|row| row.repo.as_deref())
            .collect::<Vec<_>>(),
    );
    let titles = StringArray::from(
        rows.iter()
            .map(|row| row.title.as_deref())
            .collect::<Vec<_>>(),
    );
    let scores = Float64Array::from(rows.iter().map(|row| row.score).collect::<Vec<_>>());
    let sources = StringArray::from(
        rows.iter()
            .map(|row| Some(row.source.as_str()))
            .collect::<Vec<_>>(),
    );
    let snippets = StringArray::from(
        rows.iter()
            .map(|row| row.snippet.as_deref())
            .collect::<Vec<_>>(),
    );
    let doc_types = StringArray::from(
        rows.iter()
            .map(|row| row.doc_type.as_deref())
            .collect::<Vec<_>>(),
    );
    let match_reasons = StringArray::from(
        rows.iter()
            .map(|row| row.match_reason.as_deref())
            .collect::<Vec<_>>(),
    );
    let best_sections = StringArray::from(
        rows.iter()
            .map(|row| row.best_section.as_deref())
            .collect::<Vec<_>>(),
    );
    let languages = StringArray::from(
        rows.iter()
            .map(|row| row.language.as_deref())
            .collect::<Vec<_>>(),
    );
    let lines = UInt64Array::from(rows.iter().map(|row| row.line).collect::<Vec<_>>());

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(ids) as ArrayRef,
            Arc::new(paths) as ArrayRef,
            Arc::new(repos) as ArrayRef,
            Arc::new(titles) as ArrayRef,
            Arc::new(scores) as ArrayRef,
            Arc::new(sources) as ArrayRef,
            Arc::new(snippets) as ArrayRef,
            Arc::new(doc_types) as ArrayRef,
            Arc::new(match_reasons) as ArrayRef,
            Arc::new(best_sections) as ArrayRef,
            Arc::new(languages) as ArrayRef,
            Arc::new(lines) as ArrayRef,
        ],
    )
    .map_err(|error| VectorStoreError::General(format!("build retrieval batch: {error}")))
}

/// Decode retrieval rows from a canonical Arrow record batch.
///
/// # Errors
///
/// Returns an error when one of the canonical retrieval columns is missing or
/// has an unexpected Arrow type.
pub fn retrieval_rows_from_record_batch(
    batch: &RecordBatch,
) -> Result<Vec<RetrievalRow>, VectorStoreError> {
    let ids = required_string_column(batch, RETRIEVAL_ID_COLUMN)?;
    let paths = required_string_column(batch, RETRIEVAL_PATH_COLUMN)?;
    let repos = required_string_column(batch, RETRIEVAL_REPO_COLUMN)?;
    let titles = required_string_column(batch, RETRIEVAL_TITLE_COLUMN)?;
    let scores = batch
        .column_by_name(RETRIEVAL_SCORE_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
        .ok_or_else(|| {
            VectorStoreError::General(format!(
                "missing Float64 retrieval column `{RETRIEVAL_SCORE_COLUMN}`"
            ))
        })?;
    let sources = required_string_column(batch, RETRIEVAL_SOURCE_COLUMN)?;
    let snippets = required_string_column(batch, RETRIEVAL_SNIPPET_COLUMN)?;
    let doc_types = required_string_column(batch, RETRIEVAL_DOC_TYPE_COLUMN)?;
    let match_reasons = required_string_column(batch, RETRIEVAL_MATCH_REASON_COLUMN)?;
    let best_sections = required_string_column(batch, RETRIEVAL_BEST_SECTION_COLUMN)?;
    let languages = required_string_column(batch, RETRIEVAL_LANGUAGE_COLUMN)?;
    let lines = batch
        .column_by_name(RETRIEVAL_LINE_COLUMN)
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            VectorStoreError::General(format!(
                "missing UInt64 retrieval column `{RETRIEVAL_LINE_COLUMN}`"
            ))
        })?;

    let mut rows = Vec::with_capacity(batch.num_rows());
    for row_index in 0..batch.num_rows() {
        rows.push(RetrievalRow {
            id: ids.value(row_index).to_string(),
            path: paths.value(row_index).to_string(),
            repo: (!repos.is_null(row_index)).then(|| repos.value(row_index).to_string()),
            title: (!titles.is_null(row_index)).then(|| titles.value(row_index).to_string()),
            score: (!scores.is_null(row_index)).then(|| scores.value(row_index)),
            source: sources.value(row_index).to_string(),
            snippet: (!snippets.is_null(row_index)).then(|| snippets.value(row_index).to_string()),
            doc_type: (!doc_types.is_null(row_index))
                .then(|| doc_types.value(row_index).to_string()),
            match_reason: (!match_reasons.is_null(row_index))
                .then(|| match_reasons.value(row_index).to_string()),
            best_section: (!best_sections.is_null(row_index))
                .then(|| best_sections.value(row_index).to_string()),
            language: (!languages.is_null(row_index))
                .then(|| languages.value(row_index).to_string()),
            line: (!lines.is_null(row_index)).then(|| lines.value(row_index)),
        });
    }
    Ok(rows)
}

/// Project payload columns from a retrieval batch and optionally filter by candidate id.
///
/// # Errors
///
/// Returns an error when the source batch cannot be decoded through the
/// canonical retrieval schema or when unsupported projection columns are
/// requested.
pub fn payload_fetch_record_batch(
    batch: &RecordBatch,
    columns: &[String],
    ids: Option<&BTreeSet<String>>,
) -> Result<RecordBatch, VectorStoreError> {
    let mut rows = retrieval_rows_from_record_batch(batch)?;
    if let Some(ids) = ids {
        rows.retain(|row| ids.contains(&row.id));
    }

    let selected = if columns.is_empty() {
        retrieval_result_columns()
    } else {
        validate_columns(columns)?;
        columns.to_vec()
    };

    projected_retrieval_rows_to_record_batch(&rows, &selected)
}

fn projected_retrieval_rows_to_record_batch(
    rows: &[RetrievalRow],
    columns: &[String],
) -> Result<RecordBatch, VectorStoreError> {
    let mut fields = Vec::with_capacity(columns.len());
    let mut arrays = Vec::<ArrayRef>::with_capacity(columns.len());

    for column in columns {
        let (field, array) = projected_retrieval_column(rows, column.as_str())?;
        fields.push(field);
        arrays.push(array);
    }

    RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays)
        .map_err(|error| VectorStoreError::General(format!("project retrieval batch: {error}")))
}

fn projected_retrieval_column(
    rows: &[RetrievalRow],
    column: &str,
) -> Result<(Field, ArrayRef), VectorStoreError> {
    match column {
        RETRIEVAL_ID_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_ID_COLUMN,
            false,
            rows.iter().map(|row| Some(row.id.as_str())).collect(),
        )),
        RETRIEVAL_PATH_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_PATH_COLUMN,
            false,
            rows.iter().map(|row| Some(row.path.as_str())).collect(),
        )),
        RETRIEVAL_REPO_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_REPO_COLUMN,
            true,
            rows.iter().map(|row| row.repo.as_deref()).collect(),
        )),
        RETRIEVAL_TITLE_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_TITLE_COLUMN,
            true,
            rows.iter().map(|row| row.title.as_deref()).collect(),
        )),
        RETRIEVAL_SCORE_COLUMN => Ok((
            Field::new(RETRIEVAL_SCORE_COLUMN, DataType::Float64, true),
            Arc::new(Float64Array::from(
                rows.iter().map(|row| row.score).collect::<Vec<_>>(),
            )),
        )),
        RETRIEVAL_SOURCE_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_SOURCE_COLUMN,
            false,
            rows.iter().map(|row| Some(row.source.as_str())).collect(),
        )),
        RETRIEVAL_SNIPPET_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_SNIPPET_COLUMN,
            true,
            rows.iter().map(|row| row.snippet.as_deref()).collect(),
        )),
        RETRIEVAL_DOC_TYPE_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_DOC_TYPE_COLUMN,
            true,
            rows.iter().map(|row| row.doc_type.as_deref()).collect(),
        )),
        RETRIEVAL_MATCH_REASON_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_MATCH_REASON_COLUMN,
            true,
            rows.iter().map(|row| row.match_reason.as_deref()).collect(),
        )),
        RETRIEVAL_BEST_SECTION_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_BEST_SECTION_COLUMN,
            true,
            rows.iter().map(|row| row.best_section.as_deref()).collect(),
        )),
        RETRIEVAL_LANGUAGE_COLUMN => Ok(projected_utf8_column(
            RETRIEVAL_LANGUAGE_COLUMN,
            true,
            rows.iter().map(|row| row.language.as_deref()).collect(),
        )),
        RETRIEVAL_LINE_COLUMN => Ok((
            Field::new(RETRIEVAL_LINE_COLUMN, DataType::UInt64, true),
            Arc::new(UInt64Array::from(
                rows.iter().map(|row| row.line).collect::<Vec<_>>(),
            )),
        )),
        other => Err(VectorStoreError::General(format!(
            "unsupported retrieval payload column `{other}`"
        ))),
    }
}

fn projected_utf8_column(
    name: &'static str,
    nullable: bool,
    values: Vec<Option<&str>>,
) -> (Field, ArrayRef) {
    (
        Field::new(name, DataType::Utf8, nullable),
        Arc::new(StringArray::from(values)),
    )
}

fn required_string_column<'a>(
    batch: &'a RecordBatch,
    column: &str,
) -> Result<&'a StringArray, VectorStoreError> {
    batch
        .column_by_name(column)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            VectorStoreError::General(format!("missing Utf8 retrieval column `{column}`"))
        })
}

fn validate_columns(columns: &[String]) -> Result<(), VectorStoreError> {
    for column in columns {
        if !retrieval_result_columns()
            .iter()
            .any(|candidate| candidate == column)
        {
            return Err(VectorStoreError::General(format!(
                "unsupported retrieval payload column `{column}`"
            )));
        }
    }
    Ok(())
}
