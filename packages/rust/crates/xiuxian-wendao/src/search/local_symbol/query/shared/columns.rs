use crate::search::ranking::{RetainedWindow, StreamingRerankTelemetry, trim_ranked_vec};
use std::collections::HashSet;
use xiuxian_db_store::EngineRecordBatch;

use super::ranking::{
    autocomplete_matches_prefix, autocomplete_suggestion_type, candidate_score, compare_candidates,
    compare_suggestions,
};
use super::types::{LocalSymbolCandidate, LocalSymbolSearchError};
use crate::gateway::studio::types::AutocompleteSuggestion;
use arrow::array::{Array, StringArray, StringViewArray, UInt64Array};

pub(crate) fn collect_candidates(
    table_name: &str,
    batch: &EngineRecordBatch,
    query_lower: &str,
    candidates: &mut Vec<LocalSymbolCandidate>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), LocalSymbolSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let id = string_column(batch, "id")?;
    let name = string_column(batch, "name")?;
    let name_folded = string_column(batch, "name_folded")?;
    let signature = string_column(batch, "signature")?;
    let owner_title = string_column(batch, "owner_title")?;
    let path = string_column(batch, "path")?;
    let line_start = u64_column(batch, "line_start")?;

    for row in 0..batch.num_rows() {
        let score = candidate_score(
            query_lower,
            name_folded.value(row),
            signature.value(row),
            if owner_title.is_null(row) {
                ""
            } else {
                owner_title.value(row)
            },
        );
        if score <= 0.0 {
            continue;
        }

        telemetry.observe_match();
        candidates.push(LocalSymbolCandidate {
            table_name: table_name.to_string(),
            id: id.value(row).to_string(),
            score,
            name: name.value(row).to_string(),
            path: path.value(row).to_string(),
            line_start: usize::try_from(line_start.value(row)).unwrap_or(usize::MAX),
        });
        telemetry.observe_working_set(candidates.len());
        if candidates.len() > window.threshold {
            let before_len = candidates.len();
            trim_ranked_vec(candidates, window.target, compare_candidates);
            telemetry.observe_trim(before_len, candidates.len());
        }
    }

    Ok(())
}

pub(crate) fn collect_suggestions(
    batch: &EngineRecordBatch,
    normalized_prefix: &str,
    suggestions: &mut Vec<AutocompleteSuggestion>,
    seen: &mut HashSet<String>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), LocalSymbolSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let name = string_column(batch, "name")?;
    let name_folded = string_column(batch, "name_folded")?;
    let language = string_column(batch, "language")?;
    let node_kind = string_column(batch, "node_kind")?;

    for row in 0..batch.num_rows() {
        let text = name.value(row).trim();
        if text.is_empty()
            || !autocomplete_matches_prefix(name_folded.value(row), normalized_prefix)
        {
            continue;
        }

        let dedupe_key = name_folded.value(row).to_string();
        if !seen.insert(dedupe_key) {
            continue;
        }

        telemetry.observe_match();
        suggestions.push(AutocompleteSuggestion {
            text: text.to_string(),
            suggestion_type: autocomplete_suggestion_type(
                language.value(row),
                nullable_string_value(node_kind, row),
            )
            .to_string(),
        });
        telemetry.observe_working_set(suggestions.len());
        if suggestions.len() > window.threshold {
            let before_len = suggestions.len();
            trim_ranked_vec(suggestions, window.target, compare_suggestions);
            telemetry.observe_trim(before_len, suggestions.len());
        }
    }

    Ok(())
}

fn string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, LocalSymbolSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        LocalSymbolSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;
    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }
    Err(LocalSymbolSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

fn u64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a UInt64Array, LocalSymbolSearchError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            LocalSymbolSearchError::Decode(format!("missing engine u64 column `{name}`"))
        })
}

fn nullable_string_value(array: EngineStringColumn<'_>, row: usize) -> Option<&str> {
    (!array.is_null(row)).then(|| array.value(row))
}

#[derive(Clone, Copy)]
enum EngineStringColumn<'a> {
    Utf8(&'a StringArray),
    Utf8View(&'a StringViewArray),
}

impl<'a> EngineStringColumn<'a> {
    fn value(self, row: usize) -> &'a str {
        match self {
            Self::Utf8(column) => column.value(row),
            Self::Utf8View(column) => column.value(row),
        }
    }

    fn is_null(self, row: usize) -> bool {
        match self {
            Self::Utf8(column) => column.is_null(row),
            Self::Utf8View(column) => column.is_null(row),
        }
    }
}
