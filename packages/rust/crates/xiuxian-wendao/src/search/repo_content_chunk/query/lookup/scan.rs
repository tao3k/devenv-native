use std::collections::{HashMap, HashSet};

use arrow::array::{Array, BooleanArray};
use xiuxian_vector_store::EngineRecordBatch;

use crate::search::ranking::{RetainedWindow, StreamingRerankTelemetry, trim_ranked_string_map};

use super::candidates::RepoContentChunkCandidate;
use super::error::RepoContentChunkSearchError;
use super::filters::RepoContentChunkSearchFilters;
use super::helpers::{
    candidate_path_key, compare_candidates, detail_projected_repo_content_columns,
    engine_string_column, engine_u64_column, exact_match_expression, exact_match_projection_column,
    filename_filter_expression, language_filter_expression, path_prefix_filter_expression,
    query_text_filter_expression, repo_content_detail_filter_expression,
    stage1_global_order_clause, stage1_path_rank_expression, stage1_projected_repo_content_columns,
    title_filter_expression,
};

const MIN_RETAINED_PATHS: usize = 128;
const RETAINED_PATH_MULTIPLIER: usize = 8;

pub(crate) fn build_repo_content_stage1_sql(
    table_name: &str,
    raw_needle: &str,
    query_lower: &str,
    language_filters: &HashSet<String>,
    filters: &RepoContentChunkSearchFilters,
    window: RetainedWindow,
) -> String {
    let projections = stage1_projected_repo_content_columns().join(", ");
    let predicates = [
        query_text_filter_expression(query_lower),
        language_filter_expression(language_filters),
        path_prefix_filter_expression(&filters.path_prefixes),
        filename_filter_expression(&filters.filename_filters),
        title_filter_expression(&filters.title_filters),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let exact_match = exact_match_expression(raw_needle).unwrap_or_else(|| "false".to_string());
    let path_rank = stage1_path_rank_expression(raw_needle).unwrap_or_else(|| {
        "ROW_NUMBER() OVER (PARTITION BY path ORDER BY line_number ASC)".to_string()
    });
    let where_clause =
        (!predicates.is_empty()).then(|| format!(" WHERE {}", predicates.join(" AND ")));
    let use_stage1_limit = query_lower.trim().len() < 8 && filters.tag_filters.is_empty();
    let order_clause = if use_stage1_limit {
        format!(
            " {} LIMIT {}",
            stage1_global_order_clause(),
            window.threshold
        )
    } else {
        String::new()
    };

    format!(
        "SELECT {projections}, {exact_match_column} FROM (SELECT {projections}, {exact_match} AS {exact_match_column}, {path_rank} AS candidate_rank FROM {table_name}{where_clause}) AS ranked WHERE candidate_rank = 1{order_clause}",
        exact_match_column = exact_match_projection_column(),
        where_clause = where_clause.unwrap_or_default(),
        order_clause = order_clause
    )
}

pub(crate) fn build_repo_content_detail_sql(
    table_name: &str,
    candidates: &[RepoContentChunkCandidate],
) -> Option<String> {
    let where_clause = repo_content_detail_filter_expression(candidates)?;
    let projections = detail_projected_repo_content_columns().join(", ");
    Some(format!(
        "SELECT {projections} FROM {table_name} WHERE {where_clause}",
    ))
}

pub(crate) fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(limit, RETAINED_PATH_MULTIPLIER, MIN_RETAINED_PATHS)
}

pub(crate) fn collect_candidates(
    batch: &EngineRecordBatch,
    _raw_needle: &str,
    best_by_path: &mut HashMap<String, RepoContentChunkCandidate>,
    window: RetainedWindow,
    telemetry: &mut StreamingRerankTelemetry,
) -> Result<(), RepoContentChunkSearchError> {
    telemetry.observe_batch(batch.num_rows());
    let path = engine_string_column(batch, "path")?;
    let language = engine_string_column(batch, "language")?;
    let line_number = engine_u64_column(batch, "line_number")?;
    let exact_match_column = batch
        .column_by_name(exact_match_projection_column())
        .and_then(|column| column.as_any().downcast_ref::<BooleanArray>());

    for row in 0..batch.num_rows() {
        let exact_match = exact_match_column
            .map_or_else(|| false, |column| !column.is_null(row) && column.value(row));
        telemetry.observe_match();
        let candidate = RepoContentChunkCandidate {
            path: path.value(row).to_string(),
            language: (!language.is_null(row) && !language.value(row).trim().is_empty())
                .then(|| language.value(row).to_string()),
            line_number: usize::try_from(line_number.value(row)).unwrap_or(usize::MAX),
            line_text: String::new(),
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

pub(crate) fn hydrate_candidate_line_texts(
    batch: &EngineRecordBatch,
    candidates_by_key: &mut HashMap<(String, usize), usize>,
    candidates: &mut [RepoContentChunkCandidate],
) -> Result<(), RepoContentChunkSearchError> {
    let path = engine_string_column(batch, "path")?;
    let line_number = engine_u64_column(batch, "line_number")?;
    let line_text = engine_string_column(batch, "line_text")?;

    for row in 0..batch.num_rows() {
        let candidate_key = (
            path.value(row).to_string(),
            usize::try_from(line_number.value(row)).unwrap_or(usize::MAX),
        );
        if let Some(index) = candidates_by_key.remove(&candidate_key) {
            candidates[index].line_text = if line_text.is_null(row) {
                String::new()
            } else {
                line_text.value(row).to_string()
            };
        }
    }

    Ok(())
}
