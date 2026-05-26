//! Arrow table contracts for dataset-to-ontology materialization outputs.

use crate::arrow_contract::{ArrowFieldContract, ArrowFieldType, ArrowTableContract};

/// Canonical object observation table name.
pub const DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME: &str = "ontology_object_observation";
/// Canonical link observation table name.
pub const DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME: &str = "ontology_link_observation";
/// Canonical evidence table name.
pub const DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME: &str = "ontology_evidence";
/// Compatibility entity table name used by ontology validation SQL.
pub const DATASET_ONTOLOGY_ENTITY_TABLE_NAME: &str = "ontology_entity";
/// Compatibility relation table name used by ontology validation SQL.
pub const DATASET_ONTOLOGY_RELATION_TABLE_NAME: &str = "ontology_relation";
/// Semantic object read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_OBJECTS_TABLE_NAME: &str = "semantic_objects";
/// Semantic relation read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_RELATIONS_TABLE_NAME: &str = "semantic_relations";
/// Semantic projection-state read-model table name.
pub const DATASET_ONTOLOGY_SEMANTIC_PROJECTION_STATE_TABLE_NAME: &str = "semantic_projection_state";

const DATASET_ONTOLOGY_SCHEMA_VERSION: &str = "xiuxian_wendao.dataset_ontology.v1";

const OBJECT_OBSERVATION_FIELDS: [ArrowFieldContract; 9] = [
    ArrowFieldContract::new("mapping_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("domain", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("object_type", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("rdf_class", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("object_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("display_name", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_table", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_row_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_hash", ArrowFieldType::Utf8, false),
];

const LINK_OBSERVATION_FIELDS: [ArrowFieldContract; 9] = [
    ArrowFieldContract::new("mapping_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("domain", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("link_type", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("rdf_property", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_object_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("target_object_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_table", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_row_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_hash", ArrowFieldType::Utf8, false),
];

const EVIDENCE_FIELDS: [ArrowFieldContract; 6] = [
    ArrowFieldContract::new("evidence_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("evidence_kind", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_table", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_row_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_hash", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("evidence_text", ArrowFieldType::Utf8, false),
];

const ENTITY_FIELDS: [ArrowFieldContract; 2] = [
    ArrowFieldContract::new("entity_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("class_iri", ArrowFieldType::Utf8, false),
];

const RELATION_FIELDS: [ArrowFieldContract; 3] = [
    ArrowFieldContract::new("source_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("target_id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("predicate", ArrowFieldType::Utf8, false),
];

pub(super) const fn object_observation_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.dataset_ontology.object_observation",
        DATASET_ONTOLOGY_SCHEMA_VERSION,
        DATASET_ONTOLOGY_OBJECT_OBSERVATION_TABLE_NAME,
        &OBJECT_OBSERVATION_FIELDS,
    )
}

pub(super) const fn link_observation_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.dataset_ontology.link_observation",
        DATASET_ONTOLOGY_SCHEMA_VERSION,
        DATASET_ONTOLOGY_LINK_OBSERVATION_TABLE_NAME,
        &LINK_OBSERVATION_FIELDS,
    )
}

pub(super) const fn evidence_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.dataset_ontology.evidence",
        DATASET_ONTOLOGY_SCHEMA_VERSION,
        DATASET_ONTOLOGY_EVIDENCE_TABLE_NAME,
        &EVIDENCE_FIELDS,
    )
}

pub(super) const fn entity_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.dataset_ontology.entity",
        DATASET_ONTOLOGY_SCHEMA_VERSION,
        DATASET_ONTOLOGY_ENTITY_TABLE_NAME,
        &ENTITY_FIELDS,
    )
}

pub(super) const fn relation_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.dataset_ontology.relation",
        DATASET_ONTOLOGY_SCHEMA_VERSION,
        DATASET_ONTOLOGY_RELATION_TABLE_NAME,
        &RELATION_FIELDS,
    )
}
