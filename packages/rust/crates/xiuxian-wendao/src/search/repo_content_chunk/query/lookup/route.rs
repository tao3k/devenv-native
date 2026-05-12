//! Query route for repository content chunk search hits.

use tokio::sync::OwnedSemaphorePermit;

use crate::duckdb::ParquetQueryEngine;
use crate::search::contracts::SearchHit;
use crate::search::ranking::sort_by_rank;
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::candidates::compare_candidates;
use super::error::RepoContentChunkSearchError;
use super::execution::{execute_repo_content_search, hydrate_repo_content_search_candidates};
use super::filters::RepoContentChunkSearchFilters;
use super::scan::retained_window;
/// `search_repo_content_chunks_with_filters` public function boundary for Wendao.

/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
/// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
pub async fn search_repo_content_chunks_with_filters(
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

    let Some(prepared) = prepare_repo_content_chunk_publication(service, repo_id).await? else {
        return Ok(Vec::new());
    };

    let execution = execute_repo_content_search(
        &prepared.query_engine,
        prepared.engine_table_name.as_str(),
        trimmed,
        language_filters,
        filters,
        retained_window(limit),
    )
    .await?;
    let hits = finalize_repo_content_chunk_hits(
        &prepared.query_engine,
        prepared.engine_table_name.as_str(),
        execution.candidates,
        filters,
        limit,
        repo_id,
    )
    .await?;
    service.record_query_telemetry(
        SearchCorpusKind::RepoContentChunk,
        execution
            .telemetry
            .finish(execution.source, Some(repo_id.to_string()), hits.len()),
    );
    Ok(hits)
}

struct PreparedRepoContentChunkPublication {
    _read_permit: OwnedSemaphorePermit,
    query_engine: ParquetQueryEngine,
    engine_table_name: String,
}

async fn prepare_repo_content_chunk_publication(
    service: &SearchPlaneService,
    repo_id: &str,
) -> Result<Option<PreparedRepoContentChunkPublication>, RepoContentChunkSearchError> {
    let read_permit = service.acquire_repo_search_read_permit().await?;
    let Some(publication) = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
        .await
        .and_then(|record| record.publication)
    else {
        return Ok(None);
    };
    if !publication.is_parquet_query_readable() {
        return Ok(None);
    }

    let parquet_path = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoContentChunk,
        publication.table_name.as_str(),
    );
    if !parquet_path.exists() {
        return Ok(None);
    }
    let engine_table_name = SearchPlaneService::repo_publication_engine_table_name(
        SearchCorpusKind::RepoContentChunk,
        publication.publication_id.as_str(),
    );
    #[cfg(feature = "duckdb")]
    let query_engine = service.repo_parquet_query_engine()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = service.repo_parquet_query_engine();
    query_engine
        .ensure_parquet_table_registered(engine_table_name.as_str(), parquet_path.as_path())
        .await?;

    Ok(Some(PreparedRepoContentChunkPublication {
        _read_permit: read_permit,
        query_engine,
        engine_table_name,
    }))
}

async fn finalize_repo_content_chunk_hits(
    query_engine: &ParquetQueryEngine,
    engine_table_name: &str,
    mut candidates: Vec<super::candidates::RepoContentChunkCandidate>,
    filters: &RepoContentChunkSearchFilters,
    limit: usize,
    repo_id: &str,
) -> Result<Vec<SearchHit>, RepoContentChunkSearchError> {
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);
    hydrate_repo_content_search_candidates(query_engine, engine_table_name, &mut candidates)
        .await?;
    let mut hits = candidates
        .into_iter()
        .map(|candidate| candidate.into_search_hit(repo_id))
        .collect::<Vec<_>>();
    filters.retain_matching_hits(&mut hits);
    Ok(hits)
}
