//! Provisional semantic SSOT read-model tables for bounded SQL evidence.

mod catalog;
mod guard;
mod query;
mod register;
mod rows;
mod schema;

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
pub use query::{
    query_semantic_read_model_payload, query_semantic_read_model_payload_with_engine,
    validate_semantic_read_model_query_text,
};
pub use register::{
    SEMANTIC_OBJECTS_TABLE_NAME, SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    SEMANTIC_RELATIONS_TABLE_NAME, build_semantic_read_model_rows,
    register_semantic_read_model_tables,
};
pub use rows::{
    SemanticObjectReadModelRow, SemanticProjectionStateReadModelRow, SemanticReadModelRows,
    SemanticRelationReadModelRow,
};

#[cfg(test)]
#[path = "../../tests/unit/semantic_read_model.rs"]
mod tests;
