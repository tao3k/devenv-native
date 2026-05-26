//! Dataset-ontology envelope conversion for `WendaoGraph` ontology read-model checks.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use serde_json::{Map, Value};
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaContractError, ArrowSchemaDataType,
    WENDAO_TABLE_METADATA_KEY, build_arrow_schema, validate_record_batch_schema,
};

use super::types::WendaoGraphOntologyReadModelQualityRequestBatches;

const DATASET_ONTOLOGY_ENVELOPE_RECORD_KIND_COLUMN: &str = "recordKind";
const DATASET_ONTOLOGY_ENVELOPE_TABLE_NAME_COLUMN: &str = "tableName";
const DATASET_ONTOLOGY_ENVELOPE_PAYLOAD_JSON_COLUMN: &str = "payloadJson";
const DATASET_ONTOLOGY_SEMANTIC_READ_MODEL_KIND: &str = "semantic_read_model";
const SEMANTIC_OBJECTS_TABLE: &str = "semantic_objects";
const SEMANTIC_RELATIONS_TABLE: &str = "semantic_relations";
const SEMANTIC_PROJECTION_STATE_TABLE: &str = "semantic_projection_state";
const SEMANTIC_OBJECTS_COLUMNS: [ArrowSchemaColumn; 18] = [
    ArrowSchemaColumn::new("id", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("kind", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("title", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("status", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("confidence_score", ArrowSchemaDataType::Float64),
    ArrowSchemaColumn::new("confidence_source", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("owner_count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("owners_json", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("provenance_source", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("provenance_recorded_by", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("provenance_recorded_at", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("verification_required_json", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("verification_evidence_json", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("relation_count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("source_path", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_source_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_projection_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_projection_staleness", ArrowSchemaDataType::Utf8),
];
const SEMANTIC_RELATIONS_COLUMNS: [ArrowSchemaColumn; 7] = [
    ArrowSchemaColumn::new("source", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("kind", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("target", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("source_path", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_source_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_projection_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("read_model_projection_staleness", ArrowSchemaDataType::Utf8),
];
const SEMANTIC_PROJECTION_STATE_COLUMNS: [ArrowSchemaColumn; 9] = [
    ArrowSchemaColumn::new("projection", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("status", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("source_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("current_source_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("projection_revision", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("staleness", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("source_object_count", ArrowSchemaDataType::Int64),
    ArrowSchemaColumn::new("source_objects_json", ArrowSchemaDataType::Utf8),
    ArrowSchemaColumn::new("source_path", ArrowSchemaDataType::Utf8),
];

/// Extract `WendaoGraph` quality request tables from the Gateway dataset-ontology envelope.
///
/// The Gateway envelope is the stable transport surface for dataset ontology
/// materialization. This converter intentionally accepts only compiled
/// `semantic_read_model` rows and ignores report rows, so downstream
/// `WendaoGraph` never needs to parse raw CSV fixtures, RDF, or project config.
///
/// # Errors
///
/// Returns an error when the envelope columns are missing, a semantic read-model
/// row targets an unsupported table, a payload is malformed, a required field is
/// absent, or any required read-model table is missing.
pub fn build_wendaograph_ontology_read_model_quality_request_batches_from_dataset_ontology_envelope(
    batches: &[RecordBatch],
) -> Result<WendaoGraphOntologyReadModelQualityRequestBatches, String> {
    let envelope = collect_semantic_read_model_envelope_rows(batches)?;

    Ok(WendaoGraphOntologyReadModelQualityRequestBatches::new(
        semantic_objects_batch(&envelope.objects)?,
        semantic_relations_batch(&envelope.relations)?,
        semantic_projection_state_batch(&envelope.projection_state)?,
    ))
}

struct DatasetOntologySemanticReadModelEnvelopeRows {
    objects: Vec<Map<String, Value>>,
    relations: Vec<Map<String, Value>>,
    projection_state: Vec<Map<String, Value>>,
}

fn collect_semantic_read_model_envelope_rows(
    batches: &[RecordBatch],
) -> Result<DatasetOntologySemanticReadModelEnvelopeRows, String> {
    let mut semantic_objects = Vec::new();
    let mut semantic_relations = Vec::new();
    let mut semantic_projection_state = Vec::new();

    for batch in batches {
        let record_kind_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_RECORD_KIND_COLUMN,
            "dataset ontology envelope",
        )?;
        let table_name_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_TABLE_NAME_COLUMN,
            "dataset ontology envelope",
        )?;
        let payload_json_column = string_column(
            batch,
            DATASET_ONTOLOGY_ENVELOPE_PAYLOAD_JSON_COLUMN,
            "dataset ontology envelope",
        )?;

        for row_index in 0..batch.num_rows() {
            let record_kind = record_kind_column.value(row_index);
            if record_kind != DATASET_ONTOLOGY_SEMANTIC_READ_MODEL_KIND {
                continue;
            }

            let table_name = table_name_column.value(row_index);
            let payload = payload_json_object(payload_json_column.value(row_index), table_name)?;
            match table_name {
                SEMANTIC_OBJECTS_TABLE => semantic_objects.push(payload),
                SEMANTIC_RELATIONS_TABLE => semantic_relations.push(payload),
                SEMANTIC_PROJECTION_STATE_TABLE => semantic_projection_state.push(payload),
                unsupported => {
                    return Err(format!(
                        "dataset ontology envelope contains unsupported semantic read-model table `{unsupported}`",
                    ));
                }
            }
        }
    }

    if semantic_objects.is_empty() {
        return Err("dataset ontology envelope omitted `semantic_objects` rows".to_string());
    }
    if semantic_relations.is_empty() {
        return Err("dataset ontology envelope omitted `semantic_relations` rows".to_string());
    }
    if semantic_projection_state.is_empty() {
        return Err(
            "dataset ontology envelope omitted `semantic_projection_state` rows".to_string(),
        );
    }

    Ok(DatasetOntologySemanticReadModelEnvelopeRows {
        objects: semantic_objects,
        relations: semantic_relations,
        projection_state: semantic_projection_state,
    })
}

fn string_column<'a>(
    batch: &'a RecordBatch,
    column_name: &str,
    subject: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(column_name)
        .ok_or_else(|| format!("{subject} is missing `{column_name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("{subject} column `{column_name}` must be Utf8"))
}

fn payload_json_object(payload_json: &str, table_name: &str) -> Result<Map<String, Value>, String> {
    let value = serde_json::from_str::<Value>(payload_json).map_err(|error| {
        format!(
            "dataset ontology `{table_name}` semantic read-model payload is invalid JSON: {error}"
        )
    })?;
    match value {
        Value::Object(object) => Ok(object),
        _ => Err(format!(
            "dataset ontology `{table_name}` semantic read-model payload must be a JSON object"
        )),
    }
}

fn semantic_objects_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let contract = semantic_objects_contract();
    let batch = RecordBatch::try_new(
        read_model_schema(&contract),
        vec![
            string_array(rows, "id", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "kind", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "title", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "status", SEMANTIC_OBJECTS_TABLE)?,
            float64_array(rows, "confidence_score", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "confidence_source", SEMANTIC_OBJECTS_TABLE)?,
            int64_array(rows, "owner_count", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "owners_json", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_source", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_recorded_by", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "provenance_recorded_at", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "verification_required_json", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "verification_evidence_json", SEMANTIC_OBJECTS_TABLE)?,
            int64_array(rows, "relation_count", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_OBJECTS_TABLE)?,
            string_array(rows, "read_model_source_revision", SEMANTIC_OBJECTS_TABLE)?,
            string_array(
                rows,
                "read_model_projection_revision",
                SEMANTIC_OBJECTS_TABLE,
            )?,
            string_array(
                rows,
                "read_model_projection_staleness",
                SEMANTIC_OBJECTS_TABLE,
            )?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_OBJECTS_TABLE}` request batch: {error}"))?;
    validate_read_model_batch_schema(&batch, &contract)?;
    Ok(batch)
}

fn semantic_relations_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let contract = semantic_relations_contract();
    let batch = RecordBatch::try_new(
        read_model_schema(&contract),
        vec![
            string_array(rows, "source", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "kind", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "target", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_RELATIONS_TABLE)?,
            string_array(rows, "read_model_source_revision", SEMANTIC_RELATIONS_TABLE)?,
            string_array(
                rows,
                "read_model_projection_revision",
                SEMANTIC_RELATIONS_TABLE,
            )?,
            string_array(
                rows,
                "read_model_projection_staleness",
                SEMANTIC_RELATIONS_TABLE,
            )?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_RELATIONS_TABLE}` request batch: {error}"))?;
    validate_read_model_batch_schema(&batch, &contract)?;
    Ok(batch)
}

fn semantic_projection_state_batch(rows: &[Map<String, Value>]) -> Result<RecordBatch, String> {
    let contract = semantic_projection_state_contract();
    let batch = RecordBatch::try_new(
        read_model_schema(&contract),
        vec![
            string_array(rows, "projection", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "status", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_revision", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(
                rows,
                "current_source_revision",
                SEMANTIC_PROJECTION_STATE_TABLE,
            )?,
            string_array(rows, "projection_revision", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "staleness", SEMANTIC_PROJECTION_STATE_TABLE)?,
            int64_array(rows, "source_object_count", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_objects_json", SEMANTIC_PROJECTION_STATE_TABLE)?,
            string_array(rows, "source_path", SEMANTIC_PROJECTION_STATE_TABLE)?,
        ],
    )
    .map_err(|error| format!("build `{SEMANTIC_PROJECTION_STATE_TABLE}` request batch: {error}"))?;
    validate_read_model_batch_schema(&batch, &contract)?;
    Ok(batch)
}

fn semantic_objects_contract() -> ArrowSchemaContract {
    read_model_contract(SEMANTIC_OBJECTS_TABLE, &SEMANTIC_OBJECTS_COLUMNS)
}

fn semantic_relations_contract() -> ArrowSchemaContract {
    read_model_contract(SEMANTIC_RELATIONS_TABLE, &SEMANTIC_RELATIONS_COLUMNS)
}

fn semantic_projection_state_contract() -> ArrowSchemaContract {
    read_model_contract(
        SEMANTIC_PROJECTION_STATE_TABLE,
        &SEMANTIC_PROJECTION_STATE_COLUMNS,
    )
}

fn read_model_contract(
    table_name: &'static str,
    columns: &[ArrowSchemaColumn],
) -> ArrowSchemaContract {
    ArrowSchemaContract::new(table_name, true, columns.to_vec())
}

fn read_model_schema(contract: &ArrowSchemaContract) -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        contract,
        HashMap::from([(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]),
    ))
}

fn validate_read_model_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
) -> Result<(), String> {
    validate_record_batch_schema(batch, contract)
        .map_err(|error| read_model_schema_error(contract.table_name(), &error))
}

fn read_model_schema_error(table_name: &str, error: &ArrowSchemaContractError) -> String {
    format!("build `{table_name}` request batch produced invalid schema: {error}")
}

fn string_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_string_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(StringArray::from(values)) as ArrayRef)
}

fn int64_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_i64_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Int64Array::from(values)) as ArrayRef)
}

fn float64_array(
    rows: &[Map<String, Value>],
    field_name: &str,
    table_name: &str,
) -> Result<ArrayRef, String> {
    let values = rows
        .iter()
        .map(|row| json_f64_field(row, field_name, table_name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arc::new(Float64Array::from(values)) as ArrayRef)
}

fn json_string_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<String, String> {
    row.get(field_name)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing string field `{field_name}`"
            )
        })
}

fn json_i64_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<i64, String> {
    row.get(field_name)
        .and_then(Value::as_i64)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing int64 field `{field_name}`"
            )
        })
}

fn json_f64_field(
    row: &Map<String, Value>,
    field_name: &str,
    table_name: &str,
) -> Result<f64, String> {
    row.get(field_name)
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            format!(
                "dataset ontology `{table_name}` semantic read-model row is missing float64 field `{field_name}`"
            )
        })
}
