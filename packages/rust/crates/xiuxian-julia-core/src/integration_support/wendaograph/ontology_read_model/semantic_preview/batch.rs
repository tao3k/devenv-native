use std::sync::Arc;

use arrow::array::{ArrayRef, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;

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

pub(super) fn semantic_object_batch(
    rows: &[Row],
    artifact_label: &str,
) -> Result<RecordBatch, String> {
    rows_to_batch(
        "semantic_objects",
        SEMANTIC_OBJECT_COLUMNS,
        rows,
        artifact_label,
    )
}

pub(super) fn relation_batch(rows: &[Row], artifact_label: &str) -> Result<RecordBatch, String> {
    rows_to_batch(
        "semantic_relations",
        SEMANTIC_RELATION_COLUMNS,
        rows,
        artifact_label,
    )
}

pub(super) fn projection_state_batch(
    rows: &[ProjectionStateRow],
    artifact_label: &str,
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(
        SEMANTIC_PROJECTION_STATE_COLUMNS
            .iter()
            .map(|column| Field::new(column.name, column.arrow_data_type(), false))
            .collect::<Vec<_>>(),
    ));

    RecordBatch::try_new(
        schema,
        vec![
            strings(rows.iter().map(|row| row.projection.as_str())),
            strings(rows.iter().map(|row| row.status.as_str())),
            strings(rows.iter().map(|row| row.staleness.as_str())),
            ints(rows.iter().map(|row| row.source_object_count)),
            ints(rows.iter().map(|row| row.source_relation_count)),
            ints(rows.iter().map(|row| row.source_evidence_count)),
        ],
    )
    .map_err(|error| format!("{artifact_label} build `semantic_projection_state` batch: {error}"))
}

fn rows_to_batch(
    table_name: &str,
    columns: &[Column],
    rows: &[Row],
    artifact_label: &str,
) -> Result<RecordBatch, String> {
    let schema = Arc::new(Schema::new(
        columns
            .iter()
            .map(|column| Field::new(column.name, column.arrow_data_type(), false))
            .collect::<Vec<_>>(),
    ));
    let arrays = columns
        .iter()
        .map(|column| column.array_from_rows(table_name, rows, artifact_label))
        .collect::<Result<Vec<_>, _>>()?;

    RecordBatch::try_new(schema, arrays)
        .map_err(|error| format!("{artifact_label} build `{table_name}` batch: {error}"))
}

impl Column {
    fn arrow_data_type(self) -> DataType {
        match self.data_type {
            ColumnDataType::String => DataType::Utf8,
            ColumnDataType::Int64 => DataType::Int64,
        }
    }

    fn array_from_rows(
        self,
        table_name: &str,
        rows: &[Row],
        artifact_label: &str,
    ) -> Result<ArrayRef, String> {
        match self.data_type {
            ColumnDataType::String => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| {
                        required_value(row, self.name, table_name, index + 2, artifact_label)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(strings(values.into_iter()))
            }
            ColumnDataType::Int64 => {
                let values = rows
                    .iter()
                    .enumerate()
                    .map(|(index, row)| {
                        let value =
                            required_value(row, self.name, table_name, index + 2, artifact_label)?;
                        value.parse::<i64>().map_err(|error| {
                            format!(
                                "{artifact_label} `{table_name}` row {} field `{}` must be int64: {error}",
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

fn strings<'a>(values: impl Iterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from(values.collect::<Vec<_>>()))
}

fn ints(values: impl Iterator<Item = i64>) -> ArrayRef {
    Arc::new(Int64Array::from(values.collect::<Vec<_>>()))
}
