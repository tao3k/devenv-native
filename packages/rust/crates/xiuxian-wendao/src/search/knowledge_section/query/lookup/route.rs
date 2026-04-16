use std::collections::{BTreeMap, HashMap};

use crate::duckdb::ParquetQueryEngine;
use crate::gateway::studio::types::SearchHit;
use crate::search::knowledge_section::schema::{hit_json_column, id_column, projected_columns};
use crate::search::ranking::{
    RetainedWindow, StreamingRerankSource, StreamingRerankTelemetry, sort_by_rank,
};
use crate::search::{SearchCorpusKind, SearchPlaneService};

use super::candidates::{KnowledgeCandidate, collect_candidates, retained_window};
use super::error::KnowledgeSectionSearchError;
use super::helpers::{
    compare_candidates, engine_string_column, sql_identifier, sql_string_literal,
};

#[derive(Debug)]
struct KnowledgeSearchExecution {
    candidates: Vec<KnowledgeCandidate>,
    telemetry: StreamingRerankTelemetry,
    source: StreamingRerankSource,
}

#[derive(Clone)]
struct PreparedKnowledgeRead {
    query_engine: ParquetQueryEngine,
    table_name: String,
}

/// Search knowledge-section hits using the active corpus epoch.
pub(crate) async fn search_knowledge_sections(
    service: &SearchPlaneService,
    query: &str,
    limit: usize,
) -> Result<Vec<SearchHit>, KnowledgeSectionSearchError> {
    let query_text = query.trim();
    if query_text.is_empty() {
        return Ok(Vec::new());
    }
    let prepared = prepare_knowledge_read(service).await?;

    let execution = execute_knowledge_search(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        query_text,
        retained_window(limit),
    )
    .await?;
    let mut candidates = execution.candidates;
    sort_by_rank(&mut candidates, compare_candidates);
    candidates.truncate(limit);
    let hits = decode_knowledge_hits(
        &prepared.query_engine,
        prepared.table_name.as_str(),
        candidates,
    )
    .await?;
    service.record_query_telemetry(
        SearchCorpusKind::KnowledgeSection,
        execution
            .telemetry
            .finish(execution.source, None, hits.len()),
    );
    Ok(hits)
}

fn build_knowledge_stage1_sql(table_name: &str) -> String {
    format!(
        "SELECT {columns} FROM {table_name}",
        columns = projected_columns()
            .into_iter()
            .map(sql_identifier)
            .collect::<Vec<_>>()
            .join(", "),
        table_name = sql_identifier(table_name),
    )
}

async fn execute_knowledge_search(
    engine: &ParquetQueryEngine,
    table_name: &str,
    query_text: &str,
    window: RetainedWindow,
) -> Result<KnowledgeSearchExecution, KnowledgeSectionSearchError> {
    let query_lower = query_text.to_ascii_lowercase();
    let sql = build_knowledge_stage1_sql(table_name);
    let batches = engine.query_batches(sql.as_str()).await?;
    let mut telemetry = StreamingRerankTelemetry::new(window, None, None);
    let mut best_by_path = HashMap::<String, KnowledgeCandidate>::with_capacity(window.target);

    for batch in batches {
        collect_candidates(
            &batch,
            query_text,
            query_lower.as_str(),
            &mut best_by_path,
            window,
            &mut telemetry,
        )?;
    }

    Ok(KnowledgeSearchExecution {
        candidates: best_by_path.into_values().collect(),
        telemetry,
        source: StreamingRerankSource::Scan,
    })
}

async fn decode_knowledge_hits(
    engine: &ParquetQueryEngine,
    table_name: &str,
    candidates: Vec<KnowledgeCandidate>,
) -> Result<Vec<SearchHit>, KnowledgeSectionSearchError> {
    let payloads = load_hit_payloads_by_id(engine, table_name, candidates.as_slice()).await?;
    candidates
        .into_iter()
        .map(|candidate| {
            let hit_json = payloads.get(candidate.id.as_str()).ok_or_else(|| {
                KnowledgeSectionSearchError::Decode(format!(
                    "knowledge section hydration missing payload for id `{}`",
                    candidate.id
                ))
            })?;
            let mut hit: SearchHit = serde_json::from_str(hit_json.as_str())
                .map_err(|error| KnowledgeSectionSearchError::Decode(error.to_string()))?;
            hit.score = candidate.score;
            Ok(hit)
        })
        .collect()
}

async fn load_hit_payloads_by_id(
    engine: &ParquetQueryEngine,
    table_name: &str,
    candidates: &[KnowledgeCandidate],
) -> Result<BTreeMap<String, String>, KnowledgeSectionSearchError> {
    if candidates.is_empty() {
        return Ok(BTreeMap::new());
    }

    let sql = format!(
        "SELECT {id_column}, {hit_json_column} FROM {table_name} WHERE {id_column} IN ({ids})",
        id_column = sql_identifier(id_column()),
        hit_json_column = sql_identifier(hit_json_column()),
        table_name = sql_identifier(table_name),
        ids = candidates
            .iter()
            .map(|candidate| sql_string_literal(candidate.id.as_str()))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let mut payloads = BTreeMap::new();
    let batches = engine.query_batches(sql.as_str()).await?;

    for batch in batches {
        let id = engine_string_column(&batch, id_column())?;
        let hit_json = engine_string_column(&batch, hit_json_column())?;
        for row in 0..batch.num_rows() {
            payloads.insert(id.value(row).to_string(), hit_json.value(row).to_string());
        }
    }

    Ok(payloads)
}

async fn prepare_knowledge_read(
    service: &SearchPlaneService,
) -> Result<PreparedKnowledgeRead, KnowledgeSectionSearchError> {
    let status = service
        .coordinator()
        .status_for(SearchCorpusKind::KnowledgeSection);
    let Some(active_epoch) = status.active_epoch else {
        return Err(KnowledgeSectionSearchError::NotReady);
    };

    let parquet_path =
        service.local_epoch_parquet_path(SearchCorpusKind::KnowledgeSection, active_epoch);
    if !parquet_path.exists() {
        return Err(KnowledgeSectionSearchError::NotReady);
    }
    let table_name = SearchPlaneService::local_epoch_engine_table_name(
        SearchCorpusKind::KnowledgeSection,
        active_epoch,
    );
    #[cfg(feature = "duckdb")]
    let query_engine = ParquetQueryEngine::configured()?;
    #[cfg(not(feature = "duckdb"))]
    let query_engine = ParquetQueryEngine::configured(service.datafusion_query_engine().clone());
    query_engine
        .ensure_parquet_table_registered(table_name.as_str(), parquet_path.as_path())
        .await?;

    Ok(PreparedKnowledgeRead {
        query_engine,
        table_name,
    })
}
