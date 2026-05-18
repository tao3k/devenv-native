//! Read-only SQL query execution over semantic read-model rows.

use std::path::Path;
use std::time::{Duration, Instant};

use arrow::record_batch::RecordBatch;
use sqlparser::ast::Statement as SqlStatement;
use sqlparser::dialect::DuckDbDialect;
use sqlparser::parser::Parser;
use xiuxian_wendao_parsers::semantic_ssot::{SemanticRepository, load_semantic_repository};

use crate::local_relation::{DuckDbLocalRelationEngine, LocalRelationEngine};
use crate::payload::sql_query_payload_from_record_batches;
use crate::{SqlQueryMetadata, SqlQueryPayload};

use super::register::{
    SEMANTIC_OBJECTS_TABLE_NAME, SemanticReadModelRegistration,
    register_semantic_read_model_tables_with_stats, registered_column_count,
    registered_table_names,
};

/// Execute one SQL query over the semantic read-model tables using the provided engine.
///
/// # Errors
///
/// Returns an error when semantic rows cannot be projected, relation tables
/// cannot be registered, the SQL query fails, or result payload serialization
/// fails.
pub async fn query_semantic_read_model_payload_with_engine(
    repository: &SemanticRepository,
    query_text: &str,
    query_engine: &impl LocalRelationEngine,
) -> Result<SqlQueryPayload, String> {
    validate_semantic_read_model_query_text(query_text)?;
    let registration_started_at = Instant::now();
    let registration = register_semantic_read_model_tables_with_stats(query_engine, repository)?;
    let registration_time_ms = duration_millis_u64(registration_started_at.elapsed());
    payload_from_query_engine_batches(
        query_engine,
        query_text,
        &registration,
        registration_time_ms,
    )
    .await
}

/// Load a semantic repository root and execute one SQL query over read-model tables.
///
/// # Errors
///
/// Returns an error when the semantic repository cannot be projected into
/// tables, the SQL query fails, or the result payload cannot be serialized.
pub async fn query_semantic_read_model_payload(
    semantic_root: &Path,
    query_text: &str,
) -> Result<SqlQueryPayload, String> {
    let repository = load_semantic_repository(semantic_root);
    let query_engine = DuckDbLocalRelationEngine::new_in_memory()?;
    query_semantic_read_model_payload_with_engine(&repository, query_text, &query_engine).await
}

/// Validate one semantic read-model SQL query as a read-only single statement.
///
/// # Errors
///
/// Returns an error when the query text is blank, parses as multiple
/// statements, or resolves to anything other than one read-only query
/// statement.
pub fn validate_semantic_read_model_query_text(query_text: &str) -> Result<(), String> {
    let normalized_query = query_text.trim();
    if normalized_query.is_empty() {
        return Err("semantic read-model SQL query text must not be blank".to_string());
    }

    let dialect = DuckDbDialect {};
    let mut statements = Parser::parse_sql(&dialect, normalized_query)
        .map_err(|error| format!("failed to parse semantic read-model SQL query text: {error}"))?;
    if statements.len() != 1 {
        return Err(
            "semantic read-model SQL query text must contain exactly one statement".to_string(),
        );
    }

    let statement = statements.pop().ok_or_else(|| {
        "semantic read-model SQL query text must contain exactly one statement".to_string()
    })?;
    match statement {
        SqlStatement::Query(_) => Ok(()),
        _ => Err(
            "semantic read-model SQL query text must be a read-only query statement".to_string(),
        ),
    }
}

async fn payload_from_query_engine_batches(
    query_engine: &impl LocalRelationEngine,
    query_text: &str,
    registration: &SemanticReadModelRegistration,
    registration_time_ms: u64,
) -> Result<SqlQueryPayload, String> {
    let query_started_at = Instant::now();
    let engine_batches = query_engine.query_batches(query_text).await?;
    let local_query_execution_time_ms = duration_millis_u64(query_started_at.elapsed());
    let result_row_count = engine_batches.iter().map(RecordBatch::num_rows).sum();
    let result_bytes = engine_batches_array_bytes(&engine_batches);
    let metadata = SqlQueryMetadata {
        catalog_table_name: SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
        column_catalog_table_name: String::new(),
        view_source_catalog_table_name: String::new(),
        supports_information_schema: true,
        registered_tables: registered_table_names(),
        registered_table_count: 3,
        registered_view_count: 0,
        registered_column_count: registered_column_count(),
        registered_view_source_count: 0,
        result_batch_count: engine_batches.len(),
        result_row_count,
        registered_input_bytes: Some(registration.input_bytes.into()),
        result_bytes: Some(result_bytes.into()),
        local_relation_materialization_state: query_engine
            .relation_materialization_state(SEMANTIC_OBJECTS_TABLE_NAME)
            .map(|state| state.as_str().to_string().into()),
        local_temp_storage_peak_bytes: query_engine
            .last_query_temp_storage_peak_bytes()
            .map(Into::into),
        local_relation_engine: Some(query_engine.kind().as_str().to_string()),
        duckdb_registration_strategy: query_engine
            .relation_registration_strategy(SEMANTIC_OBJECTS_TABLE_NAME)
            .map(str::to_string),
        registered_input_batch_count: Some(registration.input_batch_count),
        registered_input_row_count: Some(registration.input_row_count),
        registration_time_ms: Some(registration_time_ms.into()),
        local_query_execution_time_ms: Some(local_query_execution_time_ms.into()),
    };
    sql_query_payload_from_record_batches(metadata, &engine_batches)
}

fn duration_millis_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn engine_batches_array_bytes(batches: &[RecordBatch]) -> u64 {
    batches.iter().fold(0_u64, |total, batch| {
        total.saturating_add(u64::try_from(batch.get_array_memory_size()).unwrap_or(u64::MAX))
    })
}
