use xiuxian_vector_store::VectorStoreError;

use crate::duckdb::ParquetQueryEngine;
use crate::gateway::studio::types::ReferenceSearchHit;
use crate::search::ranking::{
    RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry, sort_by_rank,
};
use crate::search::reference_occurrence::schema::{filter_column, projected_columns};
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::candidates::{ReferenceOccurrenceCandidate, collect_candidates, compare_candidates};
use super::decode::decode_reference_hits;
use super::helpers::{sql_identifier, sql_string_literal};

const MIN_RETAINED_REFERENCE_OCCURRENCES: usize = 64;
const RETAINED_REFERENCE_OCCURRENCE_MULTIPLIER: usize = 4;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ReferenceOccurrenceSearchError {
    #[error("reference occurrence index has no published epoch")]
    NotReady,
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
    #[error("{0}")]
    Decode(String),
}

pub(crate) async fn search_reference_occurrences(
    service: &SearchPlaneService,
    query: &str,
    limit: usize,
) -> Result<Vec<ReferenceSearchHit>, ReferenceOccurrenceSearchError> {
    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return Ok(Vec::new());
    }

    let prepared = prepare_reference_occurrence_read(service).await?;
    let execution = execute_reference_occurrence_search(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        query,
        normalized_query.as_str(),
        retained_window(limit),
    )
    .await?;
    let mut candidates = execution.candidates;
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);
    let hits = decode_reference_hits(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        candidates,
    )
    .await?;
    service.record_query_telemetry(
        SearchCorpusKind::ReferenceOccurrence,
        execution
            .telemetry
            .finish(execution.source, Some("search".to_string()), hits.len()),
    );
    Ok(hits)
}

struct ReferenceOccurrenceSearchExecution {
    candidates: Vec<ReferenceOccurrenceCandidate>,
    telemetry: StreamingRerankTelemetry,
    source: StreamingRerankSource,
}

#[derive(Clone)]
struct PreparedReferenceOccurrenceRead {
    query_engine: ParquetQueryEngine,
    table_name: String,
}

async fn execute_reference_occurrence_search(
    engine: &ParquetQueryEngine,
    table_name: &str,
    query: &str,
    normalized_query: &str,
    window: RetainedWindow,
) -> Result<ReferenceOccurrenceSearchExecution, ReferenceOccurrenceSearchError> {
    let mut telemetry = StreamingRerankTelemetry::new(window, None, None);
    let mut candidates = Vec::with_capacity(window.target);
    let sql = build_reference_occurrence_stage1_sql(table_name, normalized_query);
    let batches = engine.query_batches(sql.as_str()).await?;
    for batch in batches {
        collect_candidates(&batch, query, &mut candidates, window, &mut telemetry)?;
    }
    Ok(ReferenceOccurrenceSearchExecution {
        candidates,
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

async fn prepare_reference_occurrence_read(
    service: &SearchPlaneService,
) -> Result<PreparedReferenceOccurrenceRead, ReferenceOccurrenceSearchError> {
    let status = service
        .coordinator()
        .status_for(SearchCorpusKind::ReferenceOccurrence);
    let Some(active_epoch) = status.active_epoch else {
        return Err(ReferenceOccurrenceSearchError::NotReady);
    };

    let parquet_path =
        service.local_epoch_parquet_path(SearchCorpusKind::ReferenceOccurrence, active_epoch);
    if !parquet_path.exists() {
        return Err(ReferenceOccurrenceSearchError::NotReady);
    }
    let table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::ReferenceOccurrence,
        active_epoch,
    );
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(table_name.as_str(), parquet_path.as_path())
        .await?;

    Ok(PreparedReferenceOccurrenceRead {
        query_engine,
        table_name,
    })
}

fn retained_window(limit: usize) -> RetainedWindow {
    RetainedWindow::new(
        limit,
        RETAINED_REFERENCE_OCCURRENCE_MULTIPLIER,
        MIN_RETAINED_REFERENCE_OCCURRENCES,
    )
}

fn build_reference_occurrence_stage1_sql(table_name: &str, normalized_query: &str) -> String {
    format!(
        "SELECT {columns} FROM {table_name} WHERE {filter_column} = {query_literal}",
        columns = projected_columns()
            .into_iter()
            .map(sql_identifier)
            .collect::<Vec<_>>()
            .join(", "),
        filter_column = sql_identifier(filter_column()),
        table_name = sql_identifier(table_name),
        query_literal = sql_string_literal(normalized_query),
    )
}
