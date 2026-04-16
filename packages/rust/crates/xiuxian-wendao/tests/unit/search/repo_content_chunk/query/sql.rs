use std::collections::HashSet;

use crate::search::repo_content_chunk::query::{
    RepoContentChunkSearchFilters, build_repo_content_stage1_sql,
};

#[test]
fn build_repo_content_stage1_sql_includes_sql_native_filters() {
    let sql = build_repo_content_stage1_sql(
        "repo_content_chunk_alpha_repo",
        "needle",
        "needle",
        &HashSet::from(["julia".to_string()]),
        &RepoContentChunkSearchFilters {
            path_prefixes: HashSet::from(["src/".to_string()]),
            filename_filters: HashSet::from(["BaseModelica.jl".to_string()]),
            ..RepoContentChunkSearchFilters::default()
        },
    );

    assert!(
        sql.contains("line_text_folded LIKE '%needle%' ESCAPE '\\'"),
        "{sql}"
    );
    assert!(sql.contains("language IN ('julia')"), "{sql}");
    assert!(sql.contains("path LIKE 'src/%' ESCAPE '\\'"), "{sql}");
    assert!(sql.contains("path_folded = 'basemodelica.jl'"), "{sql}");
    assert!(
        sql.contains("path_folded LIKE '%/basemodelica.jl' ESCAPE '\\'"),
        "{sql}"
    );
    assert!(
        sql.contains("SELECT path, language, line_number, line_text, exact_match FROM (SELECT"),
        "{sql}"
    );
    assert!(
        sql.contains("strpos(line_text, 'needle') > 0 AS exact_match"),
        "{sql}"
    );
    assert!(
        sql.contains("ROW_NUMBER() OVER (PARTITION BY path ORDER BY CASE WHEN (strpos(line_text, 'needle') > 0) THEN 0 ELSE 1 END, line_number ASC) AS candidate_rank"),
        "{sql}"
    );
    assert!(
        sql.contains(
            "FROM repo_content_chunk_alpha_repo WHERE line_text_folded LIKE '%needle%' ESCAPE '\\'"
        ),
        "{sql}"
    );
    assert!(!sql.contains("line_text_folded,"), "{sql}");
}

#[test]
fn build_repo_content_stage1_sql_includes_title_filters() {
    let sql = build_repo_content_stage1_sql(
        "repo_content_chunk_alpha_repo",
        "needle",
        "needle",
        &HashSet::new(),
        &RepoContentChunkSearchFilters {
            title_filters: HashSet::from(["readme".to_string()]),
            ..RepoContentChunkSearchFilters::default()
        },
    );

    assert!(
        sql.contains("path_folded LIKE '%readme%' ESCAPE '\\'"),
        "{sql}"
    );
}
