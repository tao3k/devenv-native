use arrow::array::{Array, Float64Array, Int32Array, StringArray};
use arrow::record_batch::RecordBatch;
use serde_json::{Value, json};
use xiuxian_wendao_runtime::transport::{
    REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_NAVIGATION_LINE_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, REPO_SEARCH_NAVIGATION_PATH_COLUMN,
    REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TITLE_COLUMN,
};

use crate::integration_support::search_strategy_flow_candidates::{
    SearchStrategyFlowCandidateInput, SearchStrategyFlowRepoSearchHit,
    search_strategy_flow_candidate_input_from_repo_search_hit,
};

use super::ids::repo_relative_source_path;

pub(super) fn route_receipt(route: &str, batches: &[RecordBatch]) -> Value {
    json!({
        "route": route,
        "rowCount": row_count(batches),
    })
}

pub(super) fn decoded_payload_receipt(
    route: &str,
    batches: &[RecordBatch],
    decoded_columns: &[&str],
    evidence_anchor: &str,
) -> Value {
    json!({
        "route": route,
        "rowCount": row_count(batches),
        "decodedColumns": decoded_columns,
        "evidenceAnchor": evidence_anchor,
    })
}

pub(super) fn row_count(batches: &[RecordBatch]) -> usize {
    batches.iter().map(RecordBatch::num_rows).sum()
}

pub(super) fn repo_search_batches_to_candidate_inputs(
    batches: &[RecordBatch],
) -> Vec<SearchStrategyFlowCandidateInput> {
    let mut candidates = Vec::new();
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(relative_path) = string_at(batch, REPO_SEARCH_PATH_COLUMN, row_index)
                .or_else(|| string_at(batch, REPO_SEARCH_NAVIGATION_PATH_COLUMN, row_index))
            else {
                continue;
            };
            let title = string_at(batch, REPO_SEARCH_TITLE_COLUMN, row_index);
            let best_section = string_at(batch, REPO_SEARCH_BEST_SECTION_COLUMN, row_index);
            let hit = SearchStrategyFlowRepoSearchHit {
                relative_path: relative_path.as_str(),
                title: title.as_deref(),
                best_section: best_section.as_deref(),
                line_start: usize_at(batch, REPO_SEARCH_NAVIGATION_LINE_COLUMN, row_index),
                line_end: usize_at(batch, REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, row_index),
                score: f64_at(batch, REPO_SEARCH_SCORE_COLUMN, row_index),
            };
            candidates.push(search_strategy_flow_candidate_input_from_repo_search_hit(
                &hit,
            ));
        }
    }
    candidates
}

pub(super) fn repo_relative_candidate_inputs(
    repo_id: &str,
    candidates: Vec<SearchStrategyFlowCandidateInput>,
) -> Vec<SearchStrategyFlowCandidateInput> {
    candidates
        .into_iter()
        .map(|mut candidate| {
            candidate.relative_path = repo_relative_source_path(repo_id, &candidate.relative_path);
            candidate
        })
        .collect()
}

pub(super) fn first_page_index_repo_search_row(
    batches: &[RecordBatch],
    repo_id: &str,
    preferred_source_path: Option<&str>,
) -> Option<(String, Option<String>)> {
    if let Some(preferred_source_path) = preferred_source_path {
        let preferred_source_path = repo_relative_source_path(repo_id, preferred_source_path);
        return first_page_index_repo_search_row_matching(batches, repo_id, |path| {
            path == preferred_source_path
        });
    }
    first_page_index_repo_search_row_matching(batches, repo_id, |_| true)
}

fn first_page_index_repo_search_row_matching(
    batches: &[RecordBatch],
    repo_id: &str,
    matches_path: impl Fn(&str) -> bool,
) -> Option<(String, Option<String>)> {
    for batch in batches {
        for row_index in 0..batch.num_rows() {
            let Some(path) = string_at(batch, REPO_SEARCH_PATH_COLUMN, row_index)
                .or_else(|| string_at(batch, REPO_SEARCH_NAVIGATION_PATH_COLUMN, row_index))
            else {
                continue;
            };
            let repo_relative_path = repo_relative_source_path(repo_id, path.as_str());
            if matches_path(repo_relative_path.as_str()) {
                return Some((
                    repo_relative_path,
                    string_at(batch, REPO_SEARCH_DOC_ID_COLUMN, row_index),
                ));
            }
        }
    }
    None
}

pub(super) fn first_string(batch: &RecordBatch, column: &str) -> Result<String, String> {
    let array = batch
        .column_by_name(column)
        .ok_or_else(|| format!("missing column `{column}`"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("column `{column}` should be utf8"))?;
    (0..array.len())
        .find(|index| !array.is_null(*index))
        .map(|index| array.value(index).to_owned())
        .ok_or_else(|| format!("column `{column}` should contain a non-null value"))
}

fn string_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<String> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<StringArray>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        None
    } else {
        Some(array.value(row_index).to_owned())
    }
}

fn usize_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<usize> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<Int32Array>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        return None;
    }
    usize::try_from(array.value(row_index)).ok()
}

fn f64_at(batch: &RecordBatch, column: &str, row_index: usize) -> Option<f64> {
    let array = batch
        .column_by_name(column)?
        .as_any()
        .downcast_ref::<Float64Array>()?;
    if row_index >= array.len() || array.is_null(row_index) {
        None
    } else {
        Some(array.value(row_index))
    }
}

pub(super) fn route_string<'a>(route: &'a Value, key: &str) -> Result<&'a str, String> {
    route
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("SearchStrategyFlow retrieval route missing `{key}`"))
}
