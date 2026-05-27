//! Arrow table encoders for episteme semantic read-model rows.

use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use super::{
    EpistemeError, OBJECTS_TABLE, PROJECTION_STATE_TABLE, RELATIONS_TABLE, SemanticObjectRow,
    SemanticProjectionStateRow, SemanticRelationRow,
};

pub(super) fn semantic_objects_batch(
    rows: &[SemanticObjectRow],
) -> Result<RecordBatch, EpistemeError> {
    let contract = semantic_objects_contract();
    record_batch(
        read_model_schema_ref(&contract),
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
        &contract,
    )
}

pub(super) fn semantic_relations_batch(
    rows: &[SemanticRelationRow],
) -> Result<RecordBatch, EpistemeError> {
    let contract = semantic_relations_contract();
    record_batch(
        read_model_schema_ref(&contract),
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
        &contract,
    )
}

pub(super) fn semantic_projection_state_batch(
    rows: &[SemanticProjectionStateRow],
) -> Result<RecordBatch, EpistemeError> {
    let contract = semantic_projection_state_contract();
    record_batch(
        read_model_schema_ref(&contract),
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
        &contract,
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
    schema: SchemaRef,
    columns: Vec<ArrayRef>,
    contract: &ArrowSchemaContract,
) -> Result<RecordBatch, EpistemeError> {
    let batch = RecordBatch::try_new(schema, columns).map_err(|error| {
        EpistemeError::ReadModel(format!("build `{}`: {error}", contract.table_name()))
    })?;
    validate_record_batch_schema_with_options(
        &batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| {
        EpistemeError::ReadModel(format!(
            "validate `{}` schema contract: {error}",
            contract.table_name()
        ))
    })?;
    Ok(batch)
}

fn read_model_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

fn semantic_objects_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OBJECTS_TABLE,
        true,
        vec![
            utf8_column("id"),
            utf8_column("kind"),
            utf8_column("title"),
            utf8_column("status"),
            float64_column("confidence_score"),
            utf8_column("confidence_source"),
            int64_column("owner_count"),
            utf8_column("owners_json"),
            utf8_column("provenance_source"),
            utf8_column("provenance_recorded_by"),
            utf8_column("provenance_recorded_at"),
            utf8_column("verification_required_json"),
            utf8_column("verification_evidence_json"),
            int64_column("relation_count"),
            utf8_column("source_path"),
            utf8_column("read_model_source_revision"),
            utf8_column("read_model_projection_revision"),
            utf8_column("read_model_projection_staleness"),
        ],
    )
}

fn semantic_relations_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        RELATIONS_TABLE,
        true,
        vec![
            utf8_column("source"),
            utf8_column("kind"),
            utf8_column("target"),
            utf8_column("source_path"),
            utf8_column("read_model_source_revision"),
            utf8_column("read_model_projection_revision"),
            utf8_column("read_model_projection_staleness"),
        ],
    )
}

fn semantic_projection_state_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        PROJECTION_STATE_TABLE,
        true,
        vec![
            utf8_column("projection"),
            utf8_column("status"),
            utf8_column("source_revision"),
            utf8_column("current_source_revision"),
            utf8_column("projection_revision"),
            utf8_column("staleness"),
            int64_column("source_object_count"),
            utf8_column("source_objects_json"),
            utf8_column("source_path"),
        ],
    )
}

fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

fn int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int64)
}

fn float64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Float64)
}
