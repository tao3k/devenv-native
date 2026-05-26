//! Provisional semantic SSOT read-model tables for bounded SQL evidence.

mod catalog;
mod guard;
mod materialization;
mod query;
mod register;
mod rows;
mod schema;
mod snapshot;

pub use catalog::{
    SemanticReadModelCatalog, SemanticReadModelColumnCatalog, SemanticReadModelTableCatalog,
    semantic_read_model_catalog, semantic_read_model_catalog_from_root,
};
pub use guard::{
    SEMANTIC_SQL_PROJECTION_FRESHNESS_GUARD_ID, SEMANTIC_SQL_PROJECTION_FRESHNESS_OBJECT_ID,
    SEMANTIC_SQL_PROJECTION_FRESHNESS_QUERY, SemanticProjectionFreshnessFinding,
    SemanticSqlGuardEvidence, SemanticSqlGuardStatus, run_semantic_sql_projection_freshness_guard,
    run_semantic_sql_projection_freshness_guard_with_engine,
};
pub use materialization::{
    SEMANTIC_READ_MODEL_MATERIALIZATION_PREFLIGHT_SMOKE_QUERY,
    SEMANTIC_READ_MODEL_MATERIALIZATION_REFRESH_DISCIPLINE,
    SEMANTIC_READ_MODEL_MATERIALIZATION_TARGET_ENGINE,
    SEMANTIC_READ_MODEL_PLANNED_MATERIALIZATION_STATE,
    SEMANTIC_READ_MODEL_PLANNED_REGISTRATION_STRATEGY, SEMANTIC_READ_MODEL_WRITEBACK_POLICY,
    SemanticReadModelMaterializationExecutionReport, SemanticReadModelMaterializationPlan,
    SemanticReadModelMaterializationPreflightReport, SemanticReadModelMaterializationStatus,
    SemanticReadModelMaterializationTablePlan, SemanticReadModelMaterializationTablePreflight,
    semantic_read_model_materialization_plan, semantic_read_model_materialization_plan_from_root,
    semantic_read_model_materialization_preflight,
    semantic_read_model_materialization_preflight_from_root,
    semantic_read_model_materialization_preflight_with_engine,
};
pub use query::{
    query_semantic_read_model_payload, query_semantic_read_model_payload_with_engine,
    validate_semantic_read_model_query_text,
};
pub use register::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, SemanticReadModelRecordBatches,
    build_semantic_read_model_record_batches, build_semantic_read_model_rows,
    register_semantic_read_model_tables, semantic_read_model_record_batches_from_rows,
};
pub use rows::{
    SemanticObjectReadModelRow, SemanticProjectionStateReadModelRow, SemanticReadModelRows,
    SemanticRelationReadModelRow,
};
pub(crate) use schema::{
    semantic_objects_contract, semantic_projection_state_contract, semantic_relations_contract,
};
pub use snapshot::{
    SemanticReadModelSnapshot, SemanticReadModelSnapshotCheck, SemanticReadModelTableSnapshot,
    semantic_read_model_snapshot, semantic_read_model_snapshot_check,
    semantic_read_model_snapshot_check_from_root, semantic_read_model_snapshot_from_root,
};

#[cfg(test)]
#[path = "../../tests/unit/semantic_read_model/mod.rs"]
mod tests;
