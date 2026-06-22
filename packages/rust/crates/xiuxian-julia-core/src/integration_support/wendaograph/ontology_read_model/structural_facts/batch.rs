use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, WENDAO_TABLE_METADATA_KEY,
    build_arrow_schema, validate_record_batch_schema,
};

use super::types::{Column, ColumnDataType, ProjectionStateRow, Row, required_value};

const SEMANTIC_OBJECT_COLUMNS: &[Column] = &[
    Column::string("id"),
    Column::string("kind"),
    Column::string("title"),
    Column::string("domain"),
    Column::string("evidence_id"),
    Column::string("evidence_status"),
    Column::string("target_rdf_file"),
    Column::string("review_decision"),
    Column::string("promotion_decision"),
    Column::string("reviewer_id"),
    Column::int64("relation_count"),
    Column::string("status"),
    Column::string("read_model_projection_staleness"),
];

const SEMANTIC_RELATION_COLUMNS: &[Column] = &[
    Column::string("id"),
    Column::string("kind"),
    Column::string("source"),
    Column::string("target"),
    Column::string("domain"),
    Column::string("evidence_id"),
    Column::string("evidence_status"),
    Column::string("target_rdf_file"),
    Column::string("review_decision"),
    Column::string("promotion_decision"),
    Column::string("reviewer_id"),
    Column::string("status"),
    Column::string("read_model_projection_staleness"),
];

const SEMANTIC_PROJECTION_STATE_COLUMNS: &[Column] = &[
    Column::string("projection"),
    Column::string("status"),
    Column::string("staleness"),
    Column::int64("source_object_count"),
    Column::int64("source_relation_count"),
    Column::int64("source_evidence_count"),
];

pub(super) fn semantic_object_batch(rows: &[Row]) -> Result<RecordBatch, String> {
    rows_to_batch("semantic_objects", SEMANTIC_OBJECT_COLUMNS, rows)
}

pub(super) fn relation_batch(rows: &[Row]) -> Result<RecordBatch, String> {
    rows_to_batch("semantic_relations", SEMANTIC_RELATION_COLUMNS, rows)
}

pub(super) fn projection_state_batch(rows: &[ProjectionStateRow]) -> Result<RecordBatch, String> {
    let contract = table_contract(
        "semantic_projection_state",
        SEMANTIC_PROJECTION_STATE_COLUMNS,
    );

    let batch = RecordBatch::try_new(
        schema_for_contract(&contract),
        vec![
            strings(rows.iter().map(|row| row.projection.as_str())),
            strings(rows.iter().map(|row| row.status.as_str())),
            strings(rows.iter().map(|row| row.staleness.as_str())),
            ints(rows.iter().map(|row| row.source_object_count)),
            ints(rows.iter().map(|row| row.source_relation_count)),
            ints(rows.iter().map(|row| row.source_evidence_count)),
        ],
    )
    .map_err(|error| {
        format!("build structural facts `semantic_projection_state` batch: {error}")
    })?;
    validate_batch_schema(&batch, &contract)?;
    Ok(batch)
}

fn rows_to_batch(
    table_name: &'static str,
    columns: &[Column],
    rows: &[Row],
) -> Result<RecordBatch, String> {
    let contract = table_contract(table_name, columns);
    let arrays = columns
        .iter()
        .map(|column| column.array_from_rows(table_name, rows))
        .collect::<Result<Vec<_>, _>>()?;

    let batch = RecordBatch::try_new(schema_for_contract(&contract), arrays)
        .map_err(|error| format!("build structural facts `{table_name}` batch: {error}"))?;
    validate_batch_schema(&batch, &contract)?;
    Ok(batch)
}

impl Column {
    fn arrow_schema_data_type(self) -> ArrowSchemaDataType {
        match self.data_type {
            ColumnDataType::String => ArrowSchemaDataType::Utf8,
            ColumnDataType::Int64 => ArrowSchemaDataType::Int64,
        }
    }

    fn array_from_rows(self, table_name: &str, rows: &[Row]) -> Result<ArrayRef, String> {
        match self.data_type {
            ColumnDataType::String => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| required_value(row, self.name, table_name, index + 2))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(strings(values.into_iter()))
            }
            ColumnDataType::Int64 => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| {
                        let value = required_value(row, self.name, table_name, index + 2)?;
                        value.parse::<i64>().map_err(|error| {
                            format!(
                                "structural facts `{table_name}` row {} field `{}` must be int64: {error}",
                                index + 2,
                                self.name
                            )
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ints(values.into_iter()))
            }
        }
    }
}

fn table_contract(table_name: &'static str, columns: &[Column]) -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        table_name,
        true,
        columns
            .iter()
            .copied()
            .map(|column| ArrowSchemaColumn::new(column.name, column.arrow_schema_data_type()))
            .collect(),
    )
}

fn schema_for_contract(contract: &ArrowSchemaContract) -> Arc<arrow::datatypes::Schema> {
    Arc::new(build_arrow_schema(
        contract,
        HashMap::from([(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]),
    ))
}

fn validate_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
) -> Result<(), String> {
    validate_record_batch_schema(batch, contract).map_err(|error| {
        format!(
            "build structural facts `{}` batch produced invalid schema: {error}",
            contract.table_name()
        )
    })
}

fn strings<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>()))
}

fn ints(values: impl Iterator<Item = i64>) -> ArrayRef {
    Arc::new(Int64Array::from(values.collect::<Vec<_>>()))
}
