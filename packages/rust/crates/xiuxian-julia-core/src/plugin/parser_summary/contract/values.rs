//! Julia parser-summary Arrow value readers.

use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Int32Array, Int64Array, LargeStringArray, StringArray, StringViewArray,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

use super::{
    JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN, JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN,
    JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN,
};

pub(super) fn julia_parser_summary_request_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            JULIA_PARSER_SUMMARY_REQUEST_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(JULIA_PARSER_SUMMARY_SOURCE_ID_COLUMN, DataType::Utf8, false),
        Field::new(
            JULIA_PARSER_SUMMARY_SOURCE_TEXT_COLUMN,
            DataType::Utf8,
            false,
        ),
    ]))
}

pub(super) fn column_by_name<'a>(
    batch: &'a RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<&'a dyn Array, RepoIntelligenceError> {
    Ok(batch
        .column_by_name(field_name)
        .ok_or_else(|| {
            parser_summary_contract_error(
                contract_side,
                format!("missing parser-summary column `{field_name}`"),
            )
        })?
        .as_ref())
}

pub(super) fn required_utf8_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<String>, RepoIntelligenceError> {
    let values = optional_utf8_values(batch, field_name, contract_side)?;
    values
        .into_iter()
        .enumerate()
        .map(|(row_index, value)| {
            let Some(value) = value else {
                return Err(parser_summary_contract_error(
                    contract_side,
                    format!(
                        "parser-summary column `{field_name}` must not contain null values; row {row_index} is null"
                    ),
                ));
            };
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(parser_summary_contract_error(
                    contract_side,
                    format!(
                        "parser-summary column `{field_name}` must not contain blank values; row {row_index} is blank"
                    ),
                ));
            }
            Ok(trimmed.to_string())
        })
        .collect()
}

pub(super) fn optional_utf8_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    let column = column_by_name(batch, field_name, contract_side)?;
    match column.data_type() {
        DataType::Utf8 => {
            let array = column
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Utf8"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        let value = array.value(row_index).trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                })
                .collect())
        }
        DataType::LargeUtf8 => {
            let array = column
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as LargeUtf8"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        let value = array.value(row_index).trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                })
                .collect())
        }
        DataType::Utf8View => {
            let array = column
                .as_any()
                .downcast_ref::<StringViewArray>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Utf8View"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        let value = array.value(row_index).trim();
                        if value.is_empty() {
                            None
                        } else {
                            Some(value.to_string())
                        }
                    }
                })
                .collect())
        }
        DataType::Null => Ok(vec![None; column.len()]),
        _ => Err(parser_summary_contract_error(
            contract_side,
            format!(
                "parser-summary column `{field_name}` must decode as a nullable string-compatible Arrow column"
            ),
        )),
    }
}

pub(super) fn optional_utf8_values_or_missing(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    if batch.column_by_name(field_name).is_none() {
        return Ok(vec![None; batch.num_rows()]);
    }
    optional_utf8_values(batch, field_name, contract_side)
}

pub(super) fn required_bool_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<bool>, RepoIntelligenceError> {
    let values = optional_bool_values(batch, field_name, contract_side)?;
    values
        .into_iter()
        .enumerate()
        .map(|(row_index, value)| {
            value.ok_or_else(|| {
                parser_summary_contract_error(
                    contract_side,
                    format!(
                        "parser-summary column `{field_name}` must not contain null values; row {row_index} is null"
                    ),
                )
            })
        })
        .collect()
}

pub(super) fn optional_bool_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<bool>>, RepoIntelligenceError> {
    let column = column_by_name(batch, field_name, contract_side)?;
    match column.data_type() {
        DataType::Boolean => {
            let array = column
                .as_any()
                .downcast_ref::<BooleanArray>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Boolean"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        Some(array.value(row_index))
                    }
                })
                .collect())
        }
        DataType::Null => Ok(vec![None; column.len()]),
        _ => Err(parser_summary_contract_error(
            contract_side,
            format!(
                "parser-summary column `{field_name}` must decode as a nullable Boolean Arrow column"
            ),
        )),
    }
}

pub(super) fn optional_bool_values_or_missing(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<bool>>, RepoIntelligenceError> {
    if batch.column_by_name(field_name).is_none() {
        return Ok(vec![None; batch.num_rows()]);
    }
    optional_bool_values(batch, field_name, contract_side)
}

pub(super) fn optional_int_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<i64>>, RepoIntelligenceError> {
    let column = column_by_name(batch, field_name, contract_side)?;
    match column.data_type() {
        DataType::Int32 => {
            let array = column
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Int32"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        Some(i64::from(array.value(row_index)))
                    }
                })
                .collect())
        }
        DataType::Int64 => {
            let array = column
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Int64"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        Some(array.value(row_index))
                    }
                })
                .collect())
        }
        DataType::Null => Ok(vec![None; column.len()]),
        _ => Err(parser_summary_contract_error(
            contract_side,
            format!(
                "parser-summary column `{field_name}` must decode as a nullable Int32 or Int64 Arrow column"
            ),
        )),
    }
}

pub(super) fn optional_int_values_or_missing(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<i64>>, RepoIntelligenceError> {
    if batch.column_by_name(field_name).is_none() {
        return Ok(vec![None; batch.num_rows()]);
    }
    optional_int_values(batch, field_name, contract_side)
}

pub(super) fn optional_int32_values(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<i32>>, RepoIntelligenceError> {
    let column = column_by_name(batch, field_name, contract_side)?;
    match column.data_type() {
        DataType::Int32 => {
            let array = column
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Int32"),
                    )
                })?;
            Ok((0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        None
                    } else {
                        Some(array.value(row_index))
                    }
                })
                .collect())
        }
        DataType::Int64 => {
            let array = column
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or_else(|| {
                    parser_summary_contract_error(
                        contract_side,
                        format!("parser-summary column `{field_name}` must decode as Int64"),
                    )
                })?;
            (0..array.len())
                .map(|row_index| {
                    if array.is_null(row_index) {
                        Ok(None)
                    } else {
                        i32::try_from(array.value(row_index))
                            .map(Some)
                            .map_err(|error| {
                                parser_summary_contract_error(
                                    contract_side,
                                    format!(
                                        "parser-summary column `{field_name}` row {row_index} cannot narrow Int64 to Int32: {error}"
                                    ),
                                )
                            })
                    }
                })
                .collect()
        }
        DataType::Null => Ok(vec![None; column.len()]),
        _ => Err(parser_summary_contract_error(
            contract_side,
            format!(
                "parser-summary column `{field_name}` must decode as a nullable Int32-compatible Arrow column"
            ),
        )),
    }
}

pub(super) fn optional_int32_values_or_missing(
    batch: &RecordBatch,
    field_name: &str,
    contract_side: &str,
) -> Result<Vec<Option<i32>>, RepoIntelligenceError> {
    if batch.column_by_name(field_name).is_none() {
        return Ok(vec![None; batch.num_rows()]);
    }
    optional_int32_values(batch, field_name, contract_side)
}

pub(super) fn parser_summary_request_error(message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "invalid Julia parser-summary request batch: {}",
            message.into()
        ),
    }
}

pub(super) fn parser_summary_contract_error(
    contract_side: &str,
    message: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia parser-summary {contract_side} contract violation: {}",
            message.into()
        ),
    }
}
