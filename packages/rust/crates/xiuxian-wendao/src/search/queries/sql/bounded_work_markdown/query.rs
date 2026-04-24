use std::path::Path;
use std::time::{Duration, Instant};

use super::super::execution::{
    SqlQueryMetadata, SqlQueryPayload, sql_query_payload_from_engine_batches,
};
use super::register::{
    BOUNDED_WORK_MARKDOWN_TABLE_NAME, BoundedWorkMarkdownRegistration,
    register_bounded_work_markdown_table_with_stats,
};
use crate::duckdb::{DataFusionLocalRelationEngine, LocalRelationEngine};
use xiuxian_db_store::EngineRecordBatch;

/// Execute one SQL query over the bounded-work `markdown` surface using one
/// caller-provided local relation engine.
///
/// # Errors
///
/// Returns an error when the bounded-work markdown table cannot be registered
/// into the provided engine or when the SQL statement cannot be executed or
/// serialized into the stable payload shape.
pub async fn query_bounded_work_markdown_payload_with_engine(
    root: &Path,
    query_text: &str,
    query_engine: &impl LocalRelationEngine,
) -> Result<SqlQueryPayload, String> {
    let registration_started_at = Instant::now();
    let registration = register_bounded_work_markdown_table_with_stats(query_engine, root)?;
    let registration_time_ms = duration_millis_u64(registration_started_at.elapsed());
    payload_from_query_engine_batches(
        query_engine,
        query_text,
        &registration,
        registration_time_ms,
    )
    .await
}

/// Execute one SQL query over the bounded-work `markdown` surface.
///
/// # Errors
///
/// Returns an error when the bounded-work markdown table cannot be registered
/// or when the SQL statement cannot be executed or serialized into the stable
/// payload shape.
pub async fn query_bounded_work_markdown_payload(
    root: &Path,
    query_text: &str,
) -> Result<SqlQueryPayload, String> {
    let query_engine = DataFusionLocalRelationEngine::new_with_information_schema();
    query_bounded_work_markdown_payload_with_engine(root, query_text, &query_engine).await
}

async fn payload_from_query_engine_batches(
    query_engine: &impl LocalRelationEngine,
    query_text: &str,
    registration: &BoundedWorkMarkdownRegistration,
    registration_time_ms: u64,
) -> Result<SqlQueryPayload, String> {
    let query_started_at = Instant::now();
    let engine_batches = query_engine.query_batches(query_text).await?;
    let local_query_execution_time_ms = duration_millis_u64(query_started_at.elapsed());
    let result_row_count = engine_batches
        .iter()
        .map(xiuxian_db_store::EngineRecordBatch::num_rows)
        .sum();
    let result_bytes = engine_batches_array_bytes(&engine_batches);
    let metadata = SqlQueryMetadata {
        catalog_table_name: BOUNDED_WORK_MARKDOWN_TABLE_NAME.to_string(),
        column_catalog_table_name: String::new(),
        view_source_catalog_table_name: String::new(),
        supports_information_schema: true,
        registered_tables: vec![BOUNDED_WORK_MARKDOWN_TABLE_NAME.to_string()],
        registered_table_count: 1,
        registered_view_count: 0,
        registered_column_count: 7,
        registered_view_source_count: 0,
        result_batch_count: engine_batches.len(),
        result_row_count,
        registered_input_bytes: Some(registration.input_bytes),
        result_bytes: Some(result_bytes),
        local_relation_materialization_state: query_engine
            .relation_materialization_state(BOUNDED_WORK_MARKDOWN_TABLE_NAME)
            .map(|state| state.as_str().to_string()),
        local_temp_storage_peak_bytes: query_engine.last_query_temp_storage_peak_bytes(),
        local_relation_engine: Some(query_engine.kind().as_str().to_string()),
        duckdb_registration_strategy: query_engine
            .relation_registration_strategy(BOUNDED_WORK_MARKDOWN_TABLE_NAME)
            .map(str::to_string),
        registered_input_batch_count: Some(registration.input_batch_count),
        registered_input_row_count: Some(registration.input_row_count),
        registration_time_ms: Some(registration_time_ms),
        local_query_execution_time_ms: Some(local_query_execution_time_ms),
    };
    sql_query_payload_from_engine_batches(metadata, &engine_batches)
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn engine_batches_array_bytes(batches: &[EngineRecordBatch]) -> u64 {
    batches.iter().fold(0_u64, |total, batch| {
        total.saturating_add(u64::try_from(batch.get_array_memory_size()).unwrap_or(u64::MAX))
    })
}
