//! `search::repo_search::entity` owns Wendao search repo search entity behavior.

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;

use crate::parsers::search::repo_code_query::parse_repo_code_search_query;
use crate::query_core::{
    InMemoryWendaoExplainSink, RepoRetrievalQuery, WendaoExplainEvent, WendaoOperatorKind,
    WendaoRelation, query_repo_entity_relation,
};
use crate::search::contracts::{SearchHit, StudioNavigationTarget};
use crate::search::{
    SearchCorpusKind, SearchPlaneService, SearchQueryTelemetry, SearchQueryTelemetrySource,
};

/// Search repository entity rows using a raw code-search query.
///
/// # Errors
///
/// Returns a string error when the repository entity query or hit decode fails.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub async fn search_repo_entity_hits_for_query(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    raw_query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let parsed = parse_repo_code_search_query(raw_query);
    let Some(search_term) = parsed.search_term() else {
        return Ok(Vec::new());
    };

    search_repo_entity_hits(
        search_plane,
        repo_id,
        search_term,
        &parsed.language_filters,
        &parsed.kind_filters,
        limit,
    )
    .await
}

pub(crate) async fn search_repo_entity_hits(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    search_term: &str,
    language_filters: &std::collections::HashSet<String>,
    kind_filters: &std::collections::HashSet<String>,
    limit: usize,
) -> Result<Vec<SearchHit>, String> {
    let explain_sink = Arc::new(InMemoryWendaoExplainSink::new());
    let query =
        RepoRetrievalQuery::new(repo_id, search_term, language_filters, kind_filters, limit);
    let relation = query_repo_entity_relation(search_plane, &query, Some(explain_sink.clone()))
        .await
        .map_err(|error| {
            format!("repo-search entity query failed for repo `{repo_id}`: {error}")
        })?;
    record_query_core_telemetry(
        search_plane,
        SearchCorpusKind::RepoEntity,
        repo_id,
        limit,
        explain_sink.events().as_slice(),
    );

    relation_to_search_hits(repo_id, &relation)
        .map_err(|error| format!("repo-search entity decode failed for repo `{repo_id}`: {error}"))
}

pub(crate) fn relation_to_search_hits(
    repo_id: &str,
    relation: &WendaoRelation,
) -> Result<Vec<SearchHit>, xiuxian_db_store::VectorStoreError> {
    let mut hits = Vec::new();
    for batch in relation.batches() {
        let rows = xiuxian_db_store::retrieval_rows_from_record_batch(batch)?;
        hits.extend(
            rows.iter()
                .map(|row| retrieval_row_to_search_hit(repo_id, row)),
        );
    }
    Ok(hits)
}

pub(crate) fn record_query_core_telemetry(
    search_plane: &SearchPlaneService,
    corpus: SearchCorpusKind,
    repo_id: &str,
    limit: usize,
    events: &[WendaoExplainEvent],
) {
    let Some(event) = events
        .iter()
        .rev()
        .find(|event| event.operator_kind == WendaoOperatorKind::VectorSearch)
    else {
        return;
    };

    let result_count =
        u64::try_from(event.output_row_count.unwrap_or_default()).unwrap_or(u64::MAX);
    let rows_scanned = u64::try_from(
        event
            .input_row_count
            .unwrap_or(event.output_row_count.unwrap_or_default()),
    )
    .unwrap_or(u64::MAX);
    let matched_rows = result_count;
    let working_set_budget_rows = u64::try_from(limit.max(1)).unwrap_or(u64::MAX);

    search_plane.record_query_telemetry(
        corpus,
        SearchQueryTelemetry {
            captured_at: Utc::now().to_rfc3339(),
            scope: Some(repo_id.to_string()),
            source: SearchQueryTelemetrySource::Scan,
            batch_count: 1,
            rows_scanned,
            matched_rows,
            result_count,
            batch_row_limit: None,
            recall_limit_rows: Some(u64::try_from(limit).unwrap_or(u64::MAX)),
            working_set_budget_rows,
            trim_threshold_rows: working_set_budget_rows,
            peak_working_set_rows: matched_rows,
            trim_count: 0,
            dropped_candidate_count: 0,
        },
    );
}

fn retrieval_row_to_search_hit(repo_id: &str, row: &xiuxian_db_store::RetrievalRow) -> SearchHit {
    let doc_type = row.doc_type.as_ref().map_or_else(
        || "file".to_string(),
        |doc_type| doc_type.as_str().to_string(),
    );
    let mut tags = vec![
        repo_id.to_string(),
        "code".to_string(),
        doc_type.clone(),
        format!("kind:{doc_type}"),
    ];
    if let Some(language) = row
        .language
        .clone()
        .or_else(|| infer_code_language(row.path.as_str()))
    {
        tags.push(language.clone());
        tags.push(format!("lang:{language}"));
    }
    let stem = if row.id.is_empty() {
        Path::new(row.path.as_str())
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(row.path.as_str())
            .to_string()
    } else {
        row.id.clone()
    };

    SearchHit {
        stem,
        title: row.title.clone().or_else(|| Some(row.path.clone())),
        path: row.path.clone(),
        doc_type: Some(doc_type),
        tags,
        score: row.score.unwrap_or_default(),
        best_section: row.best_section.clone().or(row.snippet.clone()),
        match_reason: row
            .match_reason
            .clone()
            .or_else(|| Some(row.source.clone())),
        hierarchical_uri: None,
        hierarchy: Some(row.path.split('/').map(str::to_string).collect::<Vec<_>>()),
        saliency_score: None,
        audit_status: None,
        verification_state: None,
        implicit_backlinks: None,
        implicit_backlink_items: None,
        navigation_target: row.line.map(|line| StudioNavigationTarget {
            path: format!("{repo_id}/{}", row.path),
            category: "repo_code".to_string(),
            project_name: Some(repo_id.to_string()),
            root_label: Some(repo_id.to_string()),
            line: usize::try_from(line).ok(),
            line_end: usize::try_from(line).ok(),
            column: None,
        }),
    }
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
