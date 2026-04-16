use crate::duckdb::ParquetQueryEngine;
use crate::gateway::studio::types::SearchHit;
use crate::search::ranking::sort_by_rank;
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::error::RepoContentChunkSearchError;
use super::execution::execute_repo_content_search;
use super::filters::RepoContentChunkSearchFilters;
use super::helpers::compare_candidates;
use super::scan::retained_window;

pub(crate) async fn search_repo_content_chunks_with_filters(
    service: &SearchPlaneService,
    repo_id: &str,
    search_term: &str,
    language_filters: &std::collections::HashSet<String>,
    filters: &RepoContentChunkSearchFilters,
    limit: usize,
) -> Result<Vec<SearchHit>, RepoContentChunkSearchError> {
    let trimmed = search_term.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let _read_permit = service.acquire_repo_search_read_permit().await?;
    let Some(publication) = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
        .await
        .and_then(|record| record.publication)
    else {
        return Ok(Vec::new());
    };
    if !publication.is_parquet_query_readable() {
        return Ok(Vec::new());
    }

    let parquet_path = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        publication.table_name.as_str(),
    );
    if !parquet_path.exists() {
        return Ok(Vec::new());
    }
    let engine_table_name = SearchPlaneService::repo_publication_engine_table_name(
        SearchCorpusKind::RepoContentChunk,
        publication.publication_id.as_str(),
    );
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(engine_table_name.as_str(), parquet_path.as_path())
        .await?;

    let execution = execute_repo_content_search(
        &query_engine,
        engine_table_name.as_str(),
        trimmed,
        language_filters,
        filters,
        retained_window(limit),
    )
    .await?;
    let mut hits = execution.candidates;
    sort_by_rank(&mut hits, compare_candidates);
    hits.truncate(limit);
    let mut hits = hits
        .into_iter()
        .map(|candidate| candidate.into_search_hit(repo_id))
        .collect::<Vec<_>>();
    filters.retain_matching_hits(&mut hits);
    service.record_query_telemetry(
        SearchCorpusKind::RepoContentChunk,
        execution
            .telemetry
            .finish(execution.source, Some(repo_id.to_string()), hits.len()),
    );
    Ok(hits)
}
