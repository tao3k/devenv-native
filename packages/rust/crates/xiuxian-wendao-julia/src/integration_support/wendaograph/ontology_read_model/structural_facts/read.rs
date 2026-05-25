use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;

use arrow::array::{Array, BooleanArray, StringArray};
use arrow::datatypes::DataType;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use super::types::{ProjectionStateRow, Row, StructuralProjectionStateRow, required_value};

pub(super) fn read_structural_rows(
    path: &Path,
    table_name: &str,
    required_columns: &[&str],
) -> Result<Vec<Row>, String> {
    let file =
        File::open(path).map_err(|error| format!("read `{}` Parquet: {error}", path.display()))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .map_err(|error| format!("open `{}` Parquet reader: {error}", path.display()))?
        .build()
        .map_err(|error| format!("build `{}` Parquet reader: {error}", path.display()))?;

    let rows = reader
        .map(|batch| {
            let batch = batch
                .map_err(|error| format!("read `{}` Parquet batch: {error}", path.display()))?;
            let columns = batch
                .schema()
                .fields()
                .iter()
                .map(|field| field.name().clone())
                .collect::<Vec<_>>();
            require_columns(table_name, &columns, required_columns)?;
            (0..batch.num_rows())
                .map(|row_index| {
                    columns
                        .iter()
                        .enumerate()
                        .map(|(column_index, column_name)| {
                            parquet_cell_to_string(
                                batch.column(column_index).as_ref(),
                                row_index,
                                table_name,
                                column_name,
                            )
                            .map(|value| (column_name.clone(), value))
                        })
                        .collect::<Result<Row, String>>()
                })
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    for (row_index, row) in rows.iter().enumerate() {
        require_nonblank_values(table_name, row_index + 2, row, required_columns)?;
    }

    if rows.is_empty() {
        return Err(format!(
            "structural facts `{table_name}` Parquet must contain at least one data row"
        ));
    }
    Ok(rows)
}

pub(super) fn read_projection_state_rows(path: &Path) -> Result<Vec<ProjectionStateRow>, String> {
    let body = fs::read_to_string(path).map_err(|error| {
        format!(
            "read `{}` structural facts projection state JSON: {error}",
            path.display()
        )
    })?;
    let structural_rows = serde_json::from_str::<Vec<StructuralProjectionStateRow>>(&body)
        .map_err(|error| {
            format!(
                "structural facts `{}` projection state JSON is invalid: {error}",
                path.display()
            )
        })?;
    if structural_rows.is_empty() {
        return Err(
            "structural facts projection state JSON must contain at least one row".to_string(),
        );
    }
    structural_rows
        .into_iter()
        .enumerate()
        .map(|(index, row)| projection_state_to_semantic(index + 1, row))
        .collect()
}

fn parquet_cell_to_string(
    array: &dyn Array,
    row_index: usize,
    table_name: &str,
    column_name: &str,
) -> Result<String, String> {
    if array.is_null(row_index) {
        return Ok(String::new());
    }
    match array.data_type() {
        DataType::Utf8 => array
            .as_any()
            .downcast_ref::<StringArray>()
            .map(|values| values.value(row_index).to_owned())
            .ok_or_else(|| {
                format!("structural facts `{table_name}` column `{column_name}` is not Utf8")
            }),
        DataType::Boolean => array
            .as_any()
            .downcast_ref::<BooleanArray>()
            .map(|values| values.value(row_index).to_string())
            .ok_or_else(|| {
                format!("structural facts `{table_name}` column `{column_name}` is not Boolean")
            }),
        other => Err(format!(
            "structural facts `{table_name}` column `{column_name}` has unsupported Parquet Arrow type {other:?}"
        )),
    }
}

fn projection_state_to_semantic(
    row_number: usize,
    row: StructuralProjectionStateRow,
) -> Result<ProjectionStateRow, String> {
    for (field_name, value) in [
        ("projection", row.projection.as_str()),
        ("status", row.status.as_str()),
        ("staleness", row.staleness.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "structural facts projection state row {row_number} has blank `{field_name}`"
            ));
        }
    }
    for (field_name, value) in [
        ("sourceObjectCount", row.source_object_count),
        ("sourceRelationCount", row.source_relation_count),
        ("sourceDocumentCount", row.source_document_count),
        ("sourceAnchorCount", row.source_anchor_count),
    ] {
        if value < 0 {
            return Err(format!(
                "structural facts projection state row {row_number} has negative `{field_name}`"
            ));
        }
    }
    Ok(ProjectionStateRow {
        projection: row.projection,
        status: row.status,
        staleness: row.staleness,
        source_object_count: row.source_object_count,
        source_relation_count: row.source_relation_count,
        source_evidence_count: row.source_document_count,
    })
}

fn require_columns(
    table_name: &str,
    columns: &[String],
    required_columns: &[&str],
) -> Result<(), String> {
    let present = columns.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing = required_columns
        .iter()
        .copied()
        .filter(|column| !present.contains(column))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "structural facts `{table_name}` is missing required column(s): {}",
            missing.join(", ")
        ))
    }
}

fn require_nonblank_values(
    table_name: &str,
    row_number: usize,
    row: &Row,
    required_columns: &[&str],
) -> Result<(), String> {
    for column in required_columns {
        let value = required_value(row, column, table_name, row_number)?;
        if value.trim().is_empty() {
            return Err(format!(
                "structural facts `{table_name}` row {row_number} has blank `{column}`"
            ));
        }
    }
    Ok(())
}
