//! Local relation registration for semantic read-model tables.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_parsers::semantic_ssot::SemanticRepository;

use crate::local_relation::LocalRelationEngine;

use super::rows::{SemanticReadModelRows, build_rows};
use super::schema::{
    build_semantic_objects_record_batch, build_semantic_projection_state_record_batch,
    build_semantic_relations_record_batch, semantic_objects_schema,
    semantic_projection_state_schema, semantic_relations_schema,
};

/// Provisional semantic object read-model table.
pub const SEMANTIC_OBJECTS_TABLE_NAME: &str = "semantic_objects";
/// Provisional semantic relation read-model table.
pub const SEMANTIC_RELATIONS_TABLE_NAME: &str = "semantic_relations";
/// Provisional semantic projection-state read-model table.
pub const SEMANTIC_PROJECTION_STATE_TABLE_NAME: &str = "semantic_projection_state";

pub(super) struct SemanticReadModelRegistration {
    pub(super) rows: SemanticReadModelRows,
    pub(super) input_batch_count: usize,
    pub(super) input_row_count: usize,
    pub(super) input_bytes: u64,
}

/// Arrow record batches for the advisory semantic read-model tables.
#[derive(Debug, Clone)]
pub struct SemanticReadModelRecordBatches {
    /// Batch for the `semantic_objects` table.
    pub objects: RecordBatch,
    /// Batch for the `semantic_relations` table.
    pub relations: RecordBatch,
    /// Batch for the `semantic_projection_state` table.
    pub projection_state: RecordBatch,
}

/// Build semantic read-model rows from one validated semantic repository.
///
/// # Errors
///
/// Returns an error when the repository validation report contains issues or
/// when row JSON metadata cannot be encoded.
pub fn build_semantic_read_model_rows(
    repository: &SemanticRepository,
) -> Result<SemanticReadModelRows, String> {
    build_rows(repository)
}

/// Build Arrow record batches for the advisory semantic read-model tables.
///
/// # Errors
///
/// Returns an error when the repository is invalid or when any read-model table
/// cannot be encoded as an Arrow record batch.
pub fn build_semantic_read_model_record_batches(
    repository: &SemanticRepository,
) -> Result<SemanticReadModelRecordBatches, String> {
    let rows = build_rows(repository)?;
    semantic_read_model_record_batches_from_rows(&rows)
}

/// Build Arrow record batches from already projected semantic read-model rows.
///
/// # Errors
///
/// Returns an error when any read-model table cannot be encoded as an Arrow
/// record batch.
pub fn semantic_read_model_record_batches_from_rows(
    rows: &SemanticReadModelRows,
) -> Result<SemanticReadModelRecordBatches, String> {
    Ok(SemanticReadModelRecordBatches {
        objects: build_semantic_objects_record_batch(&rows.objects)?,
        relations: build_semantic_relations_record_batch(&rows.relations)?,
        projection_state: build_semantic_projection_state_record_batch(&rows.projection_state)?,
    })
}

/// Register semantic read-model tables into a bounded local relation engine.
///
/// # Errors
///
/// Returns an error when the repository is invalid or when any table cannot be
/// registered into the engine.
pub fn register_semantic_read_model_tables(
    query_engine: &impl LocalRelationEngine,
    repository: &SemanticRepository,
) -> Result<SemanticReadModelRows, String> {
    Ok(register_semantic_read_model_tables_with_stats(query_engine, repository)?.rows)
}

pub(super) fn register_semantic_read_model_tables_with_stats(
    query_engine: &impl LocalRelationEngine,
    repository: &SemanticRepository,
) -> Result<SemanticReadModelRegistration, String> {
    let rows = build_rows(repository)?;
    let batches = semantic_read_model_record_batches_from_rows(&rows)?;
    let input_row_count = rows.objects.len() + rows.relations.len() + rows.projection_state.len();
    let input_bytes = batches_array_bytes(&[
        batches.objects.clone(),
        batches.relations.clone(),
        batches.projection_state.clone(),
    ]);

    query_engine.register_record_batches(
        SEMANTIC_OBJECTS_TABLE_NAME,
        semantic_objects_schema(),
        vec![batches.objects],
    )?;
    query_engine.register_record_batches(
        SEMANTIC_RELATIONS_TABLE_NAME,
        semantic_relations_schema(),
        vec![batches.relations],
    )?;
    query_engine.register_record_batches(
        SEMANTIC_PROJECTION_STATE_TABLE_NAME,
        semantic_projection_state_schema(),
        vec![batches.projection_state],
    )?;

    Ok(SemanticReadModelRegistration {
        rows,
        input_batch_count: 3,
        input_row_count,
        input_bytes,
    })
}

pub(super) fn registered_column_count() -> usize {
    semantic_objects_schema().fields().len()
        + semantic_relations_schema().fields().len()
        + semantic_projection_state_schema().fields().len()
}

pub(super) fn registered_table_names() -> Vec<String> {
    vec![
        SEMANTIC_OBJECTS_TABLE_NAME.to_string(),
        SEMANTIC_RELATIONS_TABLE_NAME.to_string(),
        SEMANTIC_PROJECTION_STATE_TABLE_NAME.to_string(),
    ]
}

fn batches_array_bytes(batches: &[RecordBatch]) -> u64 {
    batches.iter().fold(0_u64, |total, batch| {
        total.saturating_add(u64::try_from(batch.get_array_memory_size()).unwrap_or(u64::MAX))
    })
}
