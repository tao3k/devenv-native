use std::sync::Arc;

use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::rows::{
    SemanticObjectReadModelRow, SemanticProjectionStateReadModelRow, SemanticRelationReadModelRow,
};

pub(super) fn semantic_objects_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("confidence_score", DataType::Float64, false),
        Field::new("confidence_source", DataType::Utf8, false),
        Field::new("owner_count", DataType::Int64, false),
        Field::new("owners_json", DataType::Utf8, false),
        Field::new("provenance_source", DataType::Utf8, false),
        Field::new("provenance_recorded_by", DataType::Utf8, false),
        Field::new("provenance_recorded_at", DataType::Utf8, false),
        Field::new("verification_required_json", DataType::Utf8, false),
        Field::new("verification_evidence_json", DataType::Utf8, false),
        Field::new("relation_count", DataType::Int64, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("read_model_source_revision", DataType::Utf8, false),
        Field::new("read_model_projection_revision", DataType::Utf8, false),
        Field::new("read_model_projection_staleness", DataType::Utf8, false),
    ]))
}

pub(super) fn semantic_relations_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("source", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("target", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("read_model_source_revision", DataType::Utf8, false),
        Field::new("read_model_projection_revision", DataType::Utf8, false),
        Field::new("read_model_projection_staleness", DataType::Utf8, false),
    ]))
}

pub(super) fn semantic_projection_state_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("projection", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("source_revision", DataType::Utf8, false),
        Field::new("current_source_revision", DataType::Utf8, false),
        Field::new("projection_revision", DataType::Utf8, false),
        Field::new("staleness", DataType::Utf8, false),
        Field::new("source_object_count", DataType::Int64, false),
        Field::new("source_objects_json", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
    ]))
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
