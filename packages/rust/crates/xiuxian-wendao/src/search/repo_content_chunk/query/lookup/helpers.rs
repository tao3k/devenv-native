//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::HashSet;
use std::path::Path;

use arrow::array::{Array, StringArray, StringViewArray, UInt64Array};
use xiuxian_db_store::EngineRecordBatch;

use crate::search::repo_content_chunk::schema::{
    language_column, line_number_column, line_text_column, line_text_folded_column, path_column,
    path_folded_column,
};

use super::error::RepoContentChunkSearchError;

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

pub(crate) fn infer_code_language(path: &str) -> Option<String> {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("jl") || ext.eq_ignore_ascii_case("julia") => {
            Some("julia".to_string())
        }
        Some(ext) if ext.eq_ignore_ascii_case("mo") || ext.eq_ignore_ascii_case("modelica") => {
            Some("modelica".to_string())
        }
        Some(ext) if ext.eq_ignore_ascii_case("rs") => Some("rust".to_string()),
        Some(ext) if ext.eq_ignore_ascii_case("py") => Some("python".to_string()),
        Some(ext) if ext.eq_ignore_ascii_case("ts") || ext.eq_ignore_ascii_case("tsx") => {
            Some("typescript".to_string())
        }
        _ => None,
    }
}

pub(crate) fn truncate_content_search_snippet(value: &str, max_chars: usize) -> String {
    let truncated = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        format!("{truncated}...")
    } else {
        truncated
    }
}

pub(crate) fn engine_string_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<EngineStringColumn<'a>, RepoContentChunkSearchError> {
    let column = batch.column_by_name(name).ok_or_else(|| {
        RepoContentChunkSearchError::Decode(format!("missing engine string column `{name}`"))
    })?;

    if let Some(array) = column.as_any().downcast_ref::<StringArray>() {
        return Ok(EngineStringColumn::Utf8(array));
    }
    if let Some(array) = column.as_any().downcast_ref::<StringViewArray>() {
        return Ok(EngineStringColumn::Utf8View(array));
    }

    Err(RepoContentChunkSearchError::Decode(format!(
        "engine column `{name}` is not utf8-like"
    )))
}

pub(crate) fn engine_u64_column<'a>(
    batch: &'a EngineRecordBatch,
    name: &str,
) -> Result<&'a UInt64Array, RepoContentChunkSearchError> {
    batch
        .column_by_name(name)
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            RepoContentChunkSearchError::Decode(format!("missing engine u64 column `{name}`"))
        })
}

pub(crate) fn stage1_projected_repo_content_columns() -> Vec<String> {
    [path_column(), language_column(), line_number_column()]
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) fn detail_projected_repo_content_columns() -> Vec<String> {
    [path_column(), line_number_column(), line_text_column()]
        .into_iter()
        .map(str::to_string)
        .collect()
}

pub(crate) const fn exact_match_projection_column() -> &'static str {
    "exact_match"
}

pub(crate) fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn exact_match_expression(raw_needle: &str) -> Option<String> {
    let trimmed = raw_needle.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(format!(
        "strpos({column}, {value}) > 0",
        column = line_text_column(),
        value = sql_string_literal(trimmed)
    ))
}

pub(crate) fn stage1_global_order_clause() -> String {
    format!(
        "ORDER BY MIN(CASE WHEN {exact_match_column} THEN 0 ELSE 1 END) ASC, {path_column} ASC, {line_number_column} ASC",
        exact_match_column = exact_match_projection_column(),
        path_column = path_column(),
        line_number_column = line_number_column()
    )
}

pub(crate) fn language_filter_expression(language_filters: &HashSet<String>) -> Option<String> {
    if language_filters.is_empty() {
        return None;
    }

    let mut sorted = language_filters.iter().cloned().collect::<Vec<_>>();
    sorted.sort_unstable();
    Some(format!(
        "{column} IN ({values})",
        column = language_column(),
        values = sorted
            .into_iter()
            .map(|value| sql_string_literal(value.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

pub(crate) fn path_prefix_filter_expression(path_prefixes: &HashSet<String>) -> Option<String> {
    sql_or_expression(path_prefixes, |prefix| {
        format_like_expression(
            path_column(),
            format!("{}%", escape_like_pattern(prefix)).as_str(),
        )
    })
}

pub(crate) fn filename_filter_expression(filename_filters: &HashSet<String>) -> Option<String> {
    sql_or_expression(filename_filters, |filename| {
        let normalized = filename.to_ascii_lowercase();
        let exact = format!(
            "{column} = {value}",
            column = path_folded_column(),
            value = sql_string_literal(normalized.as_str())
        );
        let suffix = format_like_expression(
            path_folded_column(),
            format!("%/{}", escape_like_pattern(normalized.as_str())).as_str(),
        );
        format!("({exact} OR {suffix})")
    })
}

pub(crate) fn title_filter_expression(title_filters: &HashSet<String>) -> Option<String> {
    sql_or_expression(title_filters, |title_filter| {
        let normalized = title_filter.to_ascii_lowercase();
        format_like_expression(
            path_folded_column(),
            format!("%{}%", escape_like_pattern(normalized.as_str())).as_str(),
        )
    })
}

pub(crate) fn query_text_filter_expression(query_lower: &str) -> Option<String> {
    let trimmed = query_lower.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.len() >= 8 {
        Some(format!(
            "strpos({column}, {value}) > 0",
            column = line_text_folded_column(),
            value = sql_string_literal(trimmed)
        ))
    } else {
        Some(format_like_expression(
            line_text_folded_column(),
            format!("%{}%", escape_like_pattern(trimmed)).as_str(),
        ))
    }
}

fn format_like_expression(column: &str, pattern: &str) -> String {
    format!(
        "{column} LIKE {pattern} ESCAPE '\\'",
        pattern = sql_string_literal(pattern)
    )
}

fn escape_like_pattern(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn sql_or_expression<F>(values: &HashSet<String>, build_clause: F) -> Option<String>
where
    F: Fn(&str) -> String,
{
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.iter().map(String::as_str).collect::<Vec<_>>();
    sorted.sort_unstable();
    Some(format!(
        "({})",
        sorted
            .into_iter()
            .map(build_clause)
            .collect::<Vec<_>>()
            .join(" OR ")
    ))
}
