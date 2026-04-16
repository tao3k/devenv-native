use std::collections::HashMap;
use std::collections::HashSet;

use crate::duckdb::ParquetQueryEngine;
use crate::search::ranking::{RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry};

use super::candidates::RepoContentChunkCandidate;
use super::error::RepoContentChunkSearchError;
use super::filters::RepoContentChunkSearchFilters;
use super::scan::{
    build_repo_content_detail_sql, build_repo_content_stage1_sql, collect_candidates,
    hydrate_candidate_line_texts,
};

pub(super) struct RepoContentChunkSearchExecution {
    pub(super) candidates: Vec<RepoContentChunkCandidate>,
    pub(super) telemetry: StreamingRerankTelemetry,
    pub(super) source: StreamingRerankSource,
}

pub(super) async fn execute_repo_content_search(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    raw_needle: &str,
    language_filters: &HashSet<String>,
    filters: &RepoContentChunkSearchFilters,
    window: RetainedWindow,
) -> Result<RepoContentChunkSearchExecution, RepoContentChunkSearchError> {
    let query_lower = raw_needle.to_ascii_lowercase();
    let stage1_sql = build_repo_content_stage1_sql(
        table_name,
        raw_needle,
        query_lower.as_str(),
        language_filters,
        filters,
    );
    let batches = query_engine.query_batches(stage1_sql.as_str()).await?;
    let mut telemetry = StreamingRerankTelemetry::new(window, None, None);
    let mut best_by_path =
        HashMap::<String, RepoContentChunkCandidate>::with_capacity(window.target);

    for batch in batches {
        collect_candidates(
            &batch,
            raw_needle,
            &mut best_by_path,
            window,
            &mut telemetry,
        )?;
    }

    Ok(RepoContentChunkSearchExecution {
        candidates: best_by_path.into_values().collect(),
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

pub(super) async fn hydrate_repo_content_search_candidates(
    query_engine: &ParquetQueryEngine,
    table_name: &str,
    candidates: &mut [RepoContentChunkCandidate],
) -> Result<(), RepoContentChunkSearchError> {
    let Some(detail_sql) = build_repo_content_detail_sql(table_name, candidates) else {
        return Ok(());
    };
    let mut candidates_by_key = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| ((candidate.path.clone(), candidate.line_number), index))
        .collect::<HashMap<_, _>>();

    for batch in query_engine.query_batches(detail_sql.as_str()).await? {
        hydrate_candidate_line_texts(&batch, &mut candidates_by_key, candidates)?;
        if candidates_by_key.is_empty() {
            break;
        }
    }

    Ok(())
}
