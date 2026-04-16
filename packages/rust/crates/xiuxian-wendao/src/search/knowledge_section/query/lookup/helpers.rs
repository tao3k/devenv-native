use arrow::array::{Array, StringArray, StringViewArray};
use xiuxian_vector_store::EngineRecordBatch;

use super::candidates::KnowledgeCandidate;
use super::error::KnowledgeSectionSearchError;

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

pub(crate) fn compare_candidates(
    left: &KnowledgeCandidate,
    right: &KnowledgeCandidate,
) -> std::cmp::Ordering {
    right
        .score
        .partial_cmp(&left.score)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| left.stem.cmp(&right.stem))
}

pub(crate) fn candidate_path_key(candidate: &KnowledgeCandidate) -> String {
    candidate.path.clone()
}

pub(crate) fn score_candidate(
    query_text: &str,
    query_lower: &str,
    stem: &str,
    title: Option<&str>,
    best_section: Option<&str>,
    search_text_folded: &str,
) -> f64 {
    if title.is_some_and(|value| value == query_text) {
        return 1.0;
    }
    if title.is_some_and(|value| value.to_ascii_lowercase().contains(query_lower)) {
        return 0.95;
    }
    if best_section.is_some_and(|value| value.to_ascii_lowercase().contains(query_lower)) {
        return 0.9;
    }
    if stem.to_ascii_lowercase().contains(query_lower) {
        return 0.88;
    }
    if search_text_folded.contains(query_lower) {
        return 0.82;
    }
    0.0
}

pub(crate) fn engine_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, KnowledgeSectionSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        KnowledgeSectionSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;

    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }

    Err(KnowledgeSectionSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

pub(crate) fn nullable_value(array: EngineStringColumn<'_>, index: usize) -> Option<&str> {
    (!array.is_null(index)).then(|| array.value(index))
}

pub(crate) fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn sql_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}
