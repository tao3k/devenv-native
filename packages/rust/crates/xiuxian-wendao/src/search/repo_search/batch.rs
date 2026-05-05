use std::path::Path;
use std::sync::Arc;

use xiuxian_db_store::{
    LanceDataType, LanceField, LanceFloat64Array, LanceInt32Array, LanceListArray,
    LanceListBuilder, LanceRecordBatch, LanceSchema, LanceStringArray, LanceStringBuilder,
};
use xiuxian_wendao_runtime::transport::{
    REPO_SEARCH_BEST_SECTION_COLUMN, REPO_SEARCH_DOC_ID_COLUMN, REPO_SEARCH_HIERARCHY_COLUMN,
    REPO_SEARCH_LANGUAGE_COLUMN, REPO_SEARCH_MATCH_REASON_COLUMN,
    REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN, REPO_SEARCH_NAVIGATION_LINE_COLUMN,
    REPO_SEARCH_NAVIGATION_LINE_END_COLUMN, REPO_SEARCH_NAVIGATION_PATH_COLUMN,
    REPO_SEARCH_PATH_COLUMN, REPO_SEARCH_SCORE_COLUMN, REPO_SEARCH_TAGS_COLUMN,
    REPO_SEARCH_TITLE_COLUMN,
};

use crate::search::contracts::SearchHit;

struct RepoSearchBatchColumns<'a> {
    doc_ids: Vec<String>,
    paths: Vec<String>,
    titles: Vec<String>,
    best_sections: Vec<String>,
    match_reasons: Vec<String>,
    navigation_paths: Vec<String>,
    navigation_categories: Vec<String>,
    navigation_lines: Vec<i32>,
    navigation_line_ends: Vec<i32>,
    hierarchy_rows: Vec<&'a [String]>,
    tag_rows: Vec<&'a [String]>,
    scores: Vec<f64>,
    languages: Vec<String>,
}

impl<'a> RepoSearchBatchColumns<'a> {
    fn from_hits(hits: &'a [SearchHit]) -> Result<Self, String> {
        Ok(Self {
            doc_ids: hits
                .iter()
                .map(repo_search_doc_id_from_hit)
                .collect::<Vec<_>>(),
            paths: hits.iter().map(|hit| hit.path.clone()).collect::<Vec<_>>(),
            titles: hits
                .iter()
                .map(|hit| hit.title.clone().unwrap_or_else(|| hit.path.clone()))
                .collect::<Vec<_>>(),
            best_sections: hits
                .iter()
                .map(|hit| hit.best_section.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            match_reasons: hits
                .iter()
                .map(|hit| hit.match_reason.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            navigation_paths: hits
                .iter()
                .map(repo_search_navigation_path_from_hit)
                .collect::<Vec<_>>(),
            navigation_categories: hits
                .iter()
                .map(repo_search_navigation_category_from_hit)
                .collect::<Vec<_>>(),
            navigation_lines: hits
                .iter()
                .map(|hit| repo_search_navigation_line_from_hit(hit, "line"))
                .collect::<Result<Vec<_>, _>>()?,
            navigation_line_ends: hits
                .iter()
                .map(|hit| repo_search_navigation_line_from_hit(hit, "line_end"))
                .collect::<Result<Vec<_>, _>>()?,
            hierarchy_rows: hits
                .iter()
                .map(|hit| {
                    hit.hierarchy
                        .as_ref()
                        .map_or_else(|| &[][..], Vec::as_slice)
                })
                .collect::<Vec<_>>(),
            tag_rows: hits
                .iter()
                .map(|hit| hit.tags.as_slice())
                .collect::<Vec<_>>(),
            scores: hits.iter().map(|hit| hit.score).collect::<Vec<_>>(),
            languages: hits
                .iter()
                .map(repo_search_language_from_hit)
                .collect::<Vec<_>>(),
        })
    }
}

pub(crate) fn repo_search_batch_from_hits(hits: &[SearchHit]) -> Result<LanceRecordBatch, String> {
    build_repo_search_batch(RepoSearchBatchColumns::from_hits(hits)?)
}

fn build_repo_search_batch(
    columns: RepoSearchBatchColumns<'_>,
) -> Result<LanceRecordBatch, String> {
    LanceRecordBatch::try_new(
        Arc::new(repo_search_batch_schema()),
        vec![
            Arc::new(LanceStringArray::from(columns.doc_ids)),
            Arc::new(LanceStringArray::from(columns.paths)),
            Arc::new(LanceStringArray::from(columns.titles)),
            Arc::new(LanceStringArray::from(columns.best_sections)),
            Arc::new(LanceStringArray::from(columns.match_reasons)),
            Arc::new(LanceStringArray::from(columns.navigation_paths)),
            Arc::new(LanceStringArray::from(columns.navigation_categories)),
            Arc::new(LanceInt32Array::from(columns.navigation_lines)),
            Arc::new(LanceInt32Array::from(columns.navigation_line_ends)),
            Arc::new(build_utf8_list_array(&columns.hierarchy_rows)),
            Arc::new(build_utf8_list_array(&columns.tag_rows)),
            Arc::new(LanceFloat64Array::from(columns.scores)),
            Arc::new(LanceStringArray::from(columns.languages)),
        ],
    )
    .map_err(|error| format!("failed to build repo-search batch: {error}"))
}

fn repo_search_batch_schema() -> LanceSchema {
    LanceSchema::new(vec![
        LanceField::new(REPO_SEARCH_DOC_ID_COLUMN, LanceDataType::Utf8, false),
        LanceField::new(REPO_SEARCH_PATH_COLUMN, LanceDataType::Utf8, false),
        LanceField::new(REPO_SEARCH_TITLE_COLUMN, LanceDataType::Utf8, false),
        LanceField::new(REPO_SEARCH_BEST_SECTION_COLUMN, LanceDataType::Utf8, false),
        LanceField::new(REPO_SEARCH_MATCH_REASON_COLUMN, LanceDataType::Utf8, false),
        LanceField::new(
            REPO_SEARCH_NAVIGATION_PATH_COLUMN,
            LanceDataType::Utf8,
            false,
        ),
        LanceField::new(
            REPO_SEARCH_NAVIGATION_CATEGORY_COLUMN,
            LanceDataType::Utf8,
            false,
        ),
        LanceField::new(
            REPO_SEARCH_NAVIGATION_LINE_COLUMN,
            LanceDataType::Int32,
            false,
        ),
        LanceField::new(
            REPO_SEARCH_NAVIGATION_LINE_END_COLUMN,
            LanceDataType::Int32,
            false,
        ),
        LanceField::new(
            REPO_SEARCH_HIERARCHY_COLUMN,
            LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
            false,
        ),
        LanceField::new(
            REPO_SEARCH_TAGS_COLUMN,
            LanceDataType::List(Arc::new(LanceField::new("item", LanceDataType::Utf8, true))),
            false,
        ),
        LanceField::new(REPO_SEARCH_SCORE_COLUMN, LanceDataType::Float64, false),
        LanceField::new(REPO_SEARCH_LANGUAGE_COLUMN, LanceDataType::Utf8, false),
    ])
}

fn build_utf8_list_array(rows: &[&[String]]) -> LanceListArray {
    let mut builder = LanceListBuilder::new(LanceStringBuilder::new());
    for row in rows {
        for value in *row {
            builder.values().append_value(value);
        }
        builder.append(true);
    }
    builder.finish()
}

fn repo_search_doc_id_from_hit(hit: &SearchHit) -> String {
    let stem = hit.stem.trim();
    if stem.is_empty() {
        hit.path.clone()
    } else {
        stem.to_string()
    }
}

fn repo_search_navigation_path_from_hit(hit: &SearchHit) -> String {
    hit.navigation_target
        .as_ref()
        .map(|target| target.path.clone())
        .unwrap_or_default()
}

fn repo_search_navigation_category_from_hit(hit: &SearchHit) -> String {
    hit.navigation_target
        .as_ref()
        .map(|target| target.category.clone())
        .unwrap_or_default()
}

fn repo_search_navigation_line_from_hit(hit: &SearchHit, field_name: &str) -> Result<i32, String> {
    let line = match field_name {
        "line" => hit
            .navigation_target
            .as_ref()
            .and_then(|target| target.line),
        "line_end" => hit
            .navigation_target
            .as_ref()
            .and_then(|target| target.line_end),
        _ => {
            return Err(format!(
                "unsupported repo-search navigation field `{field_name}`"
            ));
        }
    };
    repo_search_navigation_line_to_i32(hit.path.as_str(), field_name, line)
}

fn repo_search_navigation_line_to_i32(
    path: &str,
    field_name: &str,
    line: Option<usize>,
) -> Result<i32, String> {
    line.map_or(Ok(0_i32), |value| {
        i32::try_from(value).map_err(|_| {
            format!("repo-search hit `{path}` {field_name} `{value}` exceeds i32 range")
        })
    })
}

fn repo_search_language_from_hit(hit: &SearchHit) -> String {
    hit.tags
        .iter()
        .find_map(|tag| tag.strip_prefix("lang:").map(ToString::to_string))
        .or_else(|| infer_code_language(hit.path.as_str()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn infer_code_language(path: &str) -> Option<String> {
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

#[cfg(test)]
#[path = "../../../tests/unit/search/repo_search/batch.rs"]
mod tests;
