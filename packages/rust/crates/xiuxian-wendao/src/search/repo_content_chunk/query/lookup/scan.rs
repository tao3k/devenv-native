use std::collections::{HashMap, HashSet};

use xiuxian_vector_store::EngineRecordBatch;

use crate::search::ranking::{RetainedWindow, StreamingRerankTelemetry, trim_ranked_string_map};

use super::candidates::RepoContentChunkCandidate;
use super::error::RepoContentChunkSearchError;
use super::filters::RepoContentChunkSearchFilters;
use super::helpers::{
    candidate_path_key, compare_candidates, engine_string_column, engine_u64_column,
    filename_filter_expression, language_filter_expression, path_prefix_filter_expression,
    projected_repo_content_columns, title_filter_expression,
};

const MIN_RETAINED_PATHS: usize = 128;
const RETAINED_PATH_MULTIPLIER: usize = 8;

pub(crate) fn build_repo_content_stage1_sql(
    table_name: &str,
    language_filters: &HashSet<String>,
    filters: &RepoContentChunkSearchFilters,
) -> String {
    let projections = projected_repo_content_columns().join(", ");
    let predicates = [
        language_filter_expression(language_filters),
        path_prefix_filter_expression(&filters.path_prefixes),
        filename_filter_expression(&filters.filename_filters),
        title_filter_expression(&filters.title_filters),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if predicates.is_empty() {
        return format!("SELECT {projections} FROM {table_name}");
    }

    format!(
        "SELECT {projections} FROM {table_name} WHERE {}",
        predicates.join(" AND ")
    )
}

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(limit, RETAINED_PATH_MULTIPLIER, MIN_RETAINED_PATHS)
}

pub(crate) fn collect_candidates(
    batch: &EngineRecordBatch,
    raw_needle: &str,
    needle: &str,
    best_by_path: &mut HashMap<String, RepoContentChunkCandidate>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), RepoContentChunkSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let path = engine_string_column(batch, "path")?;
    let language = engine_string_column(batch, "language")?;
    let line_number = engine_u64_column(batch, "line_number")?;
    let line_text = engine_string_column(batch, "line_text")?;
    let line_text_folded = engine_string_column(batch, "line_text_folded")?;

    for row in 0..batch.num_rows() {
        if line_text_folded.is_null(row) || !line_text_folded.value(row).contains(needle) {
            continue;
        }
        let exact_match = !line_text.is_null(row) && line_text.value(row).contains(raw_needle);
        telemetry.observe_match();
        let candidate = RepoContentChunkCandidate {
            path: path.value(row).to_string(),
            language: (!language.is_null(row) && !language.value(row).trim().is_empty())
                .then(|| language.value(row).to_string()),
            line_number: usize::try_from(line_number.value(row)).unwrap_or(usize::MAX),
            line_text: line_text.value(row).to_string(),
            score: if exact_match { 0.73 } else { 0.72 },
            exact_match,
        };

        match best_by_path.get(candidate.path.as_str()) {
            Some(existing) if existing.exact_match && !candidate.exact_match => {}
            Some(existing)
                if existing.exact_match == candidate.exact_match
                    && existing.line_number <= candidate.line_number => {}
            _ => {
                best_by_path.insert(candidate.path.clone(), candidate);
                telemetry.observe_working_set(best_by_path.len());
                if best_by_path.len() > window.threshold {
                    let before_len = best_by_path.len();
                    trim_ranked_string_map(
                        best_by_path,
                        window.target,
                        compare_candidates,
                        candidate_path_key,
                    );
                    telemetry.observe_trim(before_len, best_by_path.len());
                }
            }
        }
    }

    Ok(())
}
