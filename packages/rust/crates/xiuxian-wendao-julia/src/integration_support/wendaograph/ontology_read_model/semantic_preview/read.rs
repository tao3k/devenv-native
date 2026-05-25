use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;

use arrow::array::{Array, Int64Array, StringArray};
use arrow::datatypes::DataType;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use super::types::{Column, ProjectionStateRow, Row, required_value};

pub(super) fn read_parquet_rows(
    path: &Path,
    artifact_label: &str,
    table_name: &str,
    required_columns: &[Column],
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
            require_columns(artifact_label, table_name, &columns, required_columns)?;
            (0..batch.num_rows())
                .map(|row_index| {
                    columns
                        .iter()
                        .enumerate()
                        .map(|(column_index, column_name)| {
                            parquet_cell_to_string(
                                batch.column(column_index).as_ref(),
                                row_index,
                                artifact_label,
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
        require_nonblank_values(
            artifact_label,
            table_name,
            row_index + 2,
            row,
            required_columns,
        )?;
    }

    if rows.is_empty() {
        return Err(format!(
            "{artifact_label} `{table_name}` Parquet must contain at least one data row"
        ));
    }
    Ok(rows)
}

pub(super) fn read_projection_state_rows(
    path: &Path,
    artifact_label: &str,
) -> Result<Vec<ProjectionStateRow>, String> {
    let body = fs::read_to_string(path).map_err(|error| {
        format!(
            "read `{}` {artifact_label} projection state JSON: {error}",
            path.display()
        )
    })?;
    let rows = serde_json::from_str::<Vec<ProjectionStateRow>>(&body).map_err(|error| {
        format!(
            "{artifact_label} `{}` projection state JSON is invalid: {error}",
            path.display()
        )
    })?;
    if rows.is_empty() {
        return Err(format!(
            "{artifact_label} projection state JSON must contain at least one row"
        ));
    }
    for (index, row) in rows.iter().enumerate() {
        require_projection_nonblank(
            artifact_label,
            index + 1,
            "projection",
            row.projection.as_str(),
        )?;
        require_projection_nonblank(artifact_label, index + 1, "status", row.status.as_str())?;
        require_projection_nonblank(
            artifact_label,
            index + 1,
            "staleness",
            row.staleness.as_str(),
        )?;
        for (field_name, value) in [
            ("sourceObjectCount", row.source_object_count),
            ("sourceRelationCount", row.source_relation_count),
            ("sourceEvidenceCount", row.source_evidence_count),
        ] {
            if value < 0 {
                return Err(format!(
                    "{artifact_label} projection state row {} has negative `{field_name}`",
                    index + 1
                ));
            }
        }
    }
    Ok(rows)
}

fn parquet_cell_to_string(
    array: &dyn Array,
    row_index: usize,
    artifact_label: &str,
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
                format!("{artifact_label} `{table_name}` column `{column_name}` is not Utf8")
            }),
        DataType::Int64 => array
            .as_any()
            .downcast_ref::<Int64Array>()
            .map(|values| values.value(row_index).to_string())
            .ok_or_else(|| {
                format!("{artifact_label} `{table_name}` column `{column_name}` is not Int64")
            }),
        other => Err(format!(
            "{artifact_label} `{table_name}` column `{column_name}` has unsupported Parquet Arrow type {other:?}"
        )),
    }
}

fn require_columns(
    artifact_label: &str,
    table_name: &str,
    columns: &[String],
    required_columns: &[Column],
) -> Result<(), String> {
    let present = columns.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let missing = required_columns
        .iter()
        .filter_map(|column| (!present.contains(column.name)).then_some(column.name))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{artifact_label} `{table_name}` is missing required column(s): {}",
            missing.join(", ")
        ))
    }
}

fn require_nonblank_values(
    artifact_label: &str,
    table_name: &str,
    row_number: usize,
    row: &Row,
    required_columns: &[Column],
) -> Result<(), String> {
    for column in required_columns {
        let value = required_value(row, column.name, table_name, row_number, artifact_label)?;
        if value.trim().is_empty() {
            return Err(format!(
                "{artifact_label} `{table_name}` row {row_number} has blank `{}`",
                column.name
            ));
        }
    }
    Ok(())
}

fn require_projection_nonblank(
    artifact_label: &str,
    row_number: usize,
    field_name: &str,
    value: &str,
) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!(
            "{artifact_label} projection state row {row_number} has blank `{field_name}`"
        ));
    }
    Ok(())
}
