use std::collections::HashSet;

use crate::duckdb::ParquetQueryEngine;
use crate::search::SearchCorpusKind;
use crate::search::SearchPlaneService;
use crate::search::ranking::sort_by_rank;

use super::execution::{compare_candidates, execute_repo_entity_search, retained_window};
use super::types::{
    PreparedRepoEntityPublication, PreparedRepoEntitySearch, RepoEntityQuery, RepoEntitySearchError,
};

pub(crate) async fn prepare_repo_entity_search(
    service: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    language_filters: &HashSet<String>,
    kind_filters: &HashSet<String>,
    limit: usize,
) -> Result<Option<PreparedRepoEntitySearch>, RepoEntitySearchError> {
    let trimmed = query.trim();
    let query_lower = trimmed.to_ascii_lowercase();
    let (import_package_filter, import_module_filter) = parse_import_filters(query_lower.as_str());
    let Some(prepared_publication) = prepare_repo_entity_publication(service, repo_id).await?
    else {
        return Ok(None);
    };
    let PreparedRepoEntityPublication {
        _read_permit: read_permit,
        query_engine,
        engine_table_name,
        source_revision: _source_revision,
    } = prepared_publication;

    let query = RepoEntityQuery {
        query_lower: query_lower.as_str(),
        import_package_filter,
        import_module_filter,
        language_filters,
        kind_filters,
        window: retained_window(limit),
    };
    let execution =
        execute_repo_entity_search(&query_engine, engine_table_name.as_str(), &query).await?;
    let mut candidates = execution.candidates;
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);

    Ok(Some(PreparedRepoEntitySearch {
        _read_permit: read_permit,
        query_engine,
        engine_table_name,
        candidates,
        telemetry: execution.telemetry,
        source: execution.source,
    }))
}

pub(crate) async fn prepare_repo_entity_publication(
    service: &SearchPlaneService,
    repo_id: &str,
) -> Result<Option<PreparedRepoEntityPublication>, RepoEntitySearchError> {
    let read_permit = service.acquire_repo_search_read_permit().await?;
    let Some(publication) = service
        .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, repo_id)
        .await
        .and_then(|record| record.publication)
    else {
        return Ok(None);
    };
    if !publication.is_parquet_query_readable() {
        return Ok(None);
    }

    let parquet_path = service.repo_publication_parquet_path(
        SearchCorpusKind::RepoEntity,
        publication.table_name.as_str(),
    );
    if !parquet_path.exists() {
        return Ok(None);
    }

    let engine_table_name = SearchPlaneService::repo_publication_engine_table_name(
        SearchCorpusKind::RepoEntity,
        publication.publication_id.as_str(),
    );
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(engine_table_name.as_str(), parquet_path.as_path())
        .await?;

    Ok(Some(PreparedRepoEntityPublication {
        _read_permit: read_permit,
        query_engine,
        engine_table_name,
        source_revision: publication.source_revision,
    }))
}

fn parse_import_filters(query: &str) -> (Option<&str>, Option<&str>) {
    let mut package = None;
    let mut module = None;
    for segment in query.split(';') {
        let Some((key, value)) = segment.split_once('=') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() || value == "*" {
            continue;
        }
        match key.trim() {
            "package" => package = Some(value),
            "module" => module = Some(value),
            _ => {}
        }
    }
    (package, module)
}
