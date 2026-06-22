use std::sync::Arc;

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;

use crate::arrow_contract::{ArrowFieldContract, ArrowFieldType, ArrowTableContract};

use super::rows::{
    SemanticObjectReadModelRow, SemanticProjectionStateReadModelRow, SemanticRelationReadModelRow,
};

const SEMANTIC_READ_MODEL_SCHEMA_VERSION: &str = "xiuxian_wendao.semantic_read_model.v1";

const SEMANTIC_OBJECT_FIELDS: [ArrowFieldContract; 18] = [
    ArrowFieldContract::new("id", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("kind", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("title", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("status", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("confidence_score", ArrowFieldType::Float64, false),
    ArrowFieldContract::new("confidence_source", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("owner_count", ArrowFieldType::Int64, false),
    ArrowFieldContract::new("owners_json", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("provenance_source", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("provenance_recorded_by", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("provenance_recorded_at", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("verification_required_json", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("verification_evidence_json", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("relation_count", ArrowFieldType::Int64, false),
    ArrowFieldContract::new("source_path", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("read_model_source_revision", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new(
        "read_model_projection_revision",
        ArrowFieldType::Utf8,
        false,
    ),
    ArrowFieldContract::new(
        "read_model_projection_staleness",
        ArrowFieldType::Utf8,
        false,
    ),
];

const SEMANTIC_RELATION_FIELDS: [ArrowFieldContract; 7] = [
    ArrowFieldContract::new("source", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("kind", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("target", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_path", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("read_model_source_revision", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new(
        "read_model_projection_revision",
        ArrowFieldType::Utf8,
        false,
    ),
    ArrowFieldContract::new(
        "read_model_projection_staleness",
        ArrowFieldType::Utf8,
        false,
    ),
];

const SEMANTIC_PROJECTION_STATE_FIELDS: [ArrowFieldContract; 9] = [
    ArrowFieldContract::new("projection", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("status", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_revision", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("current_source_revision", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("projection_revision", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("staleness", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_object_count", ArrowFieldType::Int64, false),
    ArrowFieldContract::new("source_objects_json", ArrowFieldType::Utf8, false),
    ArrowFieldContract::new("source_path", ArrowFieldType::Utf8, false),
];

pub(crate) const fn semantic_objects_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.semantic_read_model.semantic_objects",
        SEMANTIC_READ_MODEL_SCHEMA_VERSION,
        "semantic_objects",
        &SEMANTIC_OBJECT_FIELDS,
    )
}

pub(crate) const fn semantic_relations_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.semantic_read_model.semantic_relations",
        SEMANTIC_READ_MODEL_SCHEMA_VERSION,
        "semantic_relations",
        &SEMANTIC_RELATION_FIELDS,
    )
}

pub(crate) const fn semantic_projection_state_contract() -> ArrowTableContract {
    ArrowTableContract::new(
        "xiuxian_wendao.semantic_read_model.semantic_projection_state",
        SEMANTIC_READ_MODEL_SCHEMA_VERSION,
        "semantic_projection_state",
        &SEMANTIC_PROJECTION_STATE_FIELDS,
    )
}

pub(super) fn semantic_objects_schema() -> SchemaRef {
    semantic_objects_contract().schema()
}

pub(super) fn semantic_relations_schema() -> SchemaRef {
    semantic_relations_contract().schema()
}

pub(super) fn semantic_projection_state_schema() -> SchemaRef {
    semantic_projection_state_contract().schema()
}

pub(super) fn build_semantic_objects_record_batch(
    rows: &[SemanticObjectReadModelRow],
) -> Result<RecordBatch, String> {
    let schema = semantic_objects_schema();
    RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.id.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.kind.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.title.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.status.as_str()),
            )),
            Arc::new(Float64Array::from_iter_values(
                rows.iter().map(|row| row.confidence_score),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.confidence_source.as_str()),
            )),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.owner_count),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.owners_json.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.provenance_source.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.provenance_recorded_by.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.provenance_recorded_at.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.verification_required_json.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.verification_evidence_json.as_str()),
            )),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.relation_count),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source_path.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_source_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_projection_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_projection_staleness.as_str()),
            )),
        ],
    )
    .map_err(|error| format!("failed to build semantic objects batch: {error}"))
}

pub(super) fn build_semantic_relations_record_batch(
    rows: &[SemanticRelationReadModelRow],
) -> Result<RecordBatch, String> {
    let schema = semantic_relations_schema();
    RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.kind.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.target.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source_path.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_source_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_projection_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter()
                    .map(|row| row.read_model_projection_staleness.as_str()),
            )),
        ],
    )
    .map_err(|error| format!("failed to build semantic relations batch: {error}"))
}

pub(super) fn build_semantic_projection_state_record_batch(
    rows: &[SemanticProjectionStateReadModelRow],
) -> Result<RecordBatch, String> {
    let schema = semantic_projection_state_schema();
    RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.projection.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.status.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.current_source_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.projection_revision.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.staleness.as_str()),
            )),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.source_object_count),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source_objects_json.as_str()),
            )),
            Arc::new(StringArray::from_iter_values(
                rows.iter().map(|row| row.source_path.as_str()),
            )),
        ],
    )
    .map_err(|error| format!("failed to build semantic projection state batch: {error}"))
}
