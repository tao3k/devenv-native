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
    let object_batch = build_semantic_objects_record_batch(&rows.objects)?;
    let relation_batch = build_semantic_relations_record_batch(&rows.relations)?;
    let projection_state_batch =
        build_semantic_projection_state_record_batch(&rows.projection_state)?;
    let input_row_count = rows.objects.len() + rows.relations.len() + rows.projection_state.len();
    let input_bytes = batches_array_bytes(&[
        object_batch.clone(),
        relation_batch.clone(),
        projection_state_batch.clone(),
    ]);

    query_engine.register_record_batches(
        SEMANTIC_OBJECTS_TABLE_NAME,
        semantic_objects_schema(),
        vec![object_batch],
    )?;
    query_engine.register_record_batches(
        SEMANTIC_RELATIONS_TABLE_NAME,
        semantic_relations_schema(),
        vec![relation_batch],
    )?;
    query_engine.register_record_batches(
        SEMANTIC_PROJECTION_STATE_TABLE_NAME,
        semantic_projection_state_schema(),
        vec![projection_state_batch],
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
