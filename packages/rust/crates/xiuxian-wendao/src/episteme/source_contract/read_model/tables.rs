//! Arrow table encoders for episteme semantic read-model rows.

use std::{collections::BTreeSet, sync::Arc};

use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

use super::{
    EpistemeError, OBJECTS_TABLE, PROJECTION_STATE_TABLE, RELATIONS_TABLE, SemanticObjectRow,
    SemanticProjectionStateRow, SemanticRelationRow,
};

pub(super) fn semantic_objects_batch(
    rows: &[SemanticObjectRow],
) -> Result<RecordBatch, EpistemeError> {
    let schema = Arc::new(Schema::new(vec![
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
    ]));
    record_batch(
        schema,
        vec![
            strings(rows.iter().map(|row| row.id.as_str())),
            strings(rows.iter().map(|row| row.kind)),
            strings(rows.iter().map(|row| row.title.as_str())),
            strings(rows.iter().map(|row| row.status)),
            Arc::new(Float64Array::from_iter_values(
                rows.iter().map(|row| row.confidence_score),
            )) as ArrayRef,
            strings(rows.iter().map(|row| row.confidence_source)),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.owner_count),
            )) as ArrayRef,
            strings(rows.iter().map(|row| row.owners_json.as_str())),
            strings(rows.iter().map(|row| row.provenance_source.as_str())),
            strings(rows.iter().map(|row| row.provenance_recorded_by)),
            strings(rows.iter().map(|row| row.provenance_recorded_at)),
            strings(
                rows.iter()
                    .map(|row| row.verification_required_json.as_str()),
            ),
            strings(
                rows.iter()
                    .map(|row| row.verification_evidence_json.as_str()),
            ),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.relation_count),
            )) as ArrayRef,
            strings(rows.iter().map(|row| row.source_path.as_str())),
            strings(
                rows.iter()
                    .map(|row| row.read_model_source_revision.as_str()),
            ),
            strings(rows.iter().map(|row| row.read_model_projection_revision)),
            strings(rows.iter().map(|row| row.read_model_projection_staleness)),
        ],
        OBJECTS_TABLE,
    )
}

pub(super) fn semantic_relations_batch(
    rows: &[SemanticRelationRow],
) -> Result<RecordBatch, EpistemeError> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("source", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("target", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("read_model_source_revision", DataType::Utf8, false),
        Field::new("read_model_projection_revision", DataType::Utf8, false),
        Field::new("read_model_projection_staleness", DataType::Utf8, false),
    ]));
    record_batch(
        schema,
        vec![
            strings(rows.iter().map(|row| row.source.as_str())),
            strings(rows.iter().map(|row| row.kind)),
            strings(rows.iter().map(|row| row.target.as_str())),
            strings(rows.iter().map(|row| row.source_path.as_str())),
            strings(
                rows.iter()
                    .map(|row| row.read_model_source_revision.as_str()),
            ),
            strings(rows.iter().map(|row| row.read_model_projection_revision)),
            strings(rows.iter().map(|row| row.read_model_projection_staleness)),
        ],
        RELATIONS_TABLE,
    )
}

pub(super) fn semantic_projection_state_batch(
    rows: &[SemanticProjectionStateRow],
) -> Result<RecordBatch, EpistemeError> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("projection", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("source_revision", DataType::Utf8, false),
        Field::new("current_source_revision", DataType::Utf8, false),
        Field::new("projection_revision", DataType::Utf8, false),
        Field::new("staleness", DataType::Utf8, false),
        Field::new("source_object_count", DataType::Int64, false),
        Field::new("source_objects_json", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
    ]));
    record_batch(
        schema,
        vec![
            strings(rows.iter().map(|row| row.projection)),
            strings(rows.iter().map(|row| row.status)),
            strings(rows.iter().map(|row| row.source_revision.as_str())),
            strings(rows.iter().map(|row| row.current_source_revision.as_str())),
            strings(rows.iter().map(|row| row.projection_revision)),
            strings(rows.iter().map(|row| row.staleness)),
            Arc::new(Int64Array::from_iter_values(
                rows.iter().map(|row| row.source_object_count),
            )) as ArrayRef,
            strings(rows.iter().map(|row| row.source_objects_json.as_str())),
            strings(rows.iter().map(|row| row.source_path.as_str())),
        ],
        PROJECTION_STATE_TABLE,
    )
}

pub(super) fn object_ids(rows: &RecordBatch) -> BTreeSet<String> {
    rows.column_by_name("id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .map(|array| {
            (0..array.len())
                .map(|index| array.value(index).to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn strings<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

fn record_batch(
    schema: Arc<Schema>,
    columns: Vec<ArrayRef>,
    table_name: &str,
) -> Result<RecordBatch, EpistemeError> {
    RecordBatch::try_new(schema, columns)
        .map_err(|error| EpistemeError::ReadModel(format!("build `{table_name}`: {error}")))
}
