//! Dataset-to-ontology materialization helpers for bounded SQL evidence.

mod materialization;
mod schema;
mod sql;

pub use materialization::{
    DatasetOntologyMappingSql, DatasetOntologyMaterializationReport,
    DatasetOntologyMaterializedTableCount, DatasetOntologySelectSql, DatasetOntologySourceTable,
    DatasetOntologyValidationFailure, DatasetOntologyValidationRule,
    materialize_dataset_ontology_with_engine,
};
pub use schema::{
    DATASET_ONTOLOGY_ENTITY_TABLE_NAME, DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME,
    DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME, DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
    DATASET_ONTOLOGY_RELATION_TABLE_NAME, DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME,
    DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME,
};
pub use sql::validate_dataset_ontology_select_only_sql;

#[cfg(all(test, feature = "duckdb"))]
#[path = "../../tests/unit/dataset_ontology/mod.rs"]
mod tests;
