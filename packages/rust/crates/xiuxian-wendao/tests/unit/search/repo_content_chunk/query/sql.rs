use std::collections::HashSet;

use crate::search::repo_content_chunk::query::{
    RepoContentChunkCandidate, RepoContentChunkSearchFilters, build_repo_content_detail_sql,
    build_repo_content_stage1_sql, retained_window,
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
        retained_window(5),
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
        sql.contains("SELECT path, language, line_number, exact_match FROM (SELECT"),
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
    assert!(
        sql.contains(
            "ORDER BY CASE WHEN exact_match THEN 0 ELSE 1 END, path ASC, line_number ASC LIMIT 256"
        ),
        "{sql}"
    );
    assert!(
        !sql.contains("SELECT path, language, line_number, line_text_folded"),
        "{sql}"
    );
    assert!(
        !sql.contains("SELECT path, language, line_number, line_text"),
        "{sql}"
    );
}

#[test]
fn build_repo_content_stage1_sql_uses_strpos_for_long_query_tokens() {
    let sql = build_repo_content_stage1_sql(
        "repo_content_chunk_alpha_repo",
        "needle_token",
        "needle_token",
        &HashSet::new(),
        &RepoContentChunkSearchFilters::default(),
        retained_window(5),
    );

    assert!(
        sql.contains("strpos(line_text_folded, 'needle_token') > 0"),
        "{sql}"
    );
    assert!(!sql.contains(" LIMIT "), "{sql}");
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
        retained_window(5),
    );

    assert!(
        sql.contains("path_folded LIKE '%readme%' ESCAPE '\\'"),
        "{sql}"
    );
}

#[test]
fn build_repo_content_stage1_sql_skips_sql_limit_when_tag_filters_need_post_filtering() {
    let sql = build_repo_content_stage1_sql(
        "repo_content_chunk_alpha_repo",
        "needle",
        "needle",
        &HashSet::new(),
        &RepoContentChunkSearchFilters {
            tag_filters: HashSet::from(["match:exact".to_string()]),
            ..RepoContentChunkSearchFilters::default()
        },
        retained_window(5),
    );

    assert!(!sql.contains(" LIMIT "), "{sql}");
    assert!(!sql.contains("ORDER BY CASE WHEN exact_match"), "{sql}");
}

#[test]
fn build_repo_content_detail_sql_targets_specific_path_line_pairs() {
    let sql = build_repo_content_detail_sql(
        "repo_content_chunk_alpha_repo",
        &[
            RepoContentChunkCandidate {
                path: "src/alpha.jl".to_string(),
                language: Some("julia".to_string()),
                line_number: 7,
                line_text: String::new(),
                score: 0.73,
                exact_match: true,
            },
            RepoContentChunkCandidate {
                path: "src/beta.jl".to_string(),
                language: Some("julia".to_string()),
                line_number: 11,
                line_text: String::new(),
                score: 0.72,
                exact_match: false,
            },
        ],
    )
    .expect("detail sql");

    assert!(
        sql.contains("SELECT path, line_number, line_text FROM repo_content_chunk_alpha_repo"),
        "{sql}"
    );
    assert!(
        sql.contains("(path = 'src/alpha.jl' AND line_number = 7)"),
        "{sql}"
    );
    assert!(
        sql.contains("(path = 'src/beta.jl' AND line_number = 11)"),
        "{sql}"
    );
}
