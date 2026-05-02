//! Modelica AST-query Arrow value readers.

use std::sync::Arc;

use arrow::array::{
    Array, BooleanArray, Int32Array, Int64Array, LargeStringArray, StringArray, StringViewArray,
};
use arrow::datatypes::DataType;
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::repo_intelligence::RepoIntelligenceError;

pub(super) fn required_utf8_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<String>, RepoIntelligenceError> {
    let array = batch.column_by_name(column).ok_or_else(|| {
        ast_query_contract_error(stage, format!("missing required column `{column}`"))
    })?;
    let values = utf8_values(array, column, stage)?;
    if values.iter().any(Option::is_none) {
        return Err(ast_query_contract_error(
            stage,
            format!("required column `{column}` contains null rows"),
        ));
    }
    Ok(values
        .into_iter()
        .map(Option::unwrap_or_default)
        .collect::<Vec<_>>())
}

pub(super) fn optional_utf8_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    match batch.column_by_name(column) {
        Some(array) => utf8_values(array, column, stage),
        None => Ok(vec![None; batch.num_rows()]),
    }
}

pub(super) fn utf8_values(
    array: &Arc<dyn Array>,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<String>>, RepoIntelligenceError> {
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    if let Some(values) = array.as_any().downcast_ref::<StringArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<LargeStringArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<StringViewArray>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index).to_string()))
            .collect());
    }
    Err(ast_query_contract_error(
        stage,
        format!(
            "column `{column}` expected Utf8-compatible values but found {:?}",
            array.data_type()
        ),
    ))
}

pub(super) fn required_bool_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<bool>, RepoIntelligenceError> {
    let array = batch.column_by_name(column).ok_or_else(|| {
        ast_query_contract_error(stage, format!("missing required column `{column}`"))
    })?;
    let values = array
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| {
            ast_query_contract_error(
                stage,
                format!(
                    "column `{column}` expected Boolean values but found {:?}",
                    array.data_type()
                ),
            )
        })?;
    if (0..values.len()).any(|index| values.is_null(index)) {
        return Err(ast_query_contract_error(
            stage,
            format!("required column `{column}` contains null rows"),
        ));
    }
    Ok((0..values.len()).map(|index| values.value(index)).collect())
}

pub(super) fn optional_bool_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<bool>>, RepoIntelligenceError> {
    let Some(array) = batch.column_by_name(column) else {
        return Ok(vec![None; batch.num_rows()]);
    };
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    let values = array
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| {
            ast_query_contract_error(
                stage,
                format!(
                    "column `{column}` expected Boolean values but found {:?}",
                    array.data_type()
                ),
            )
        })?;
    Ok((0..values.len())
        .map(|index| (!values.is_null(index)).then(|| values.value(index)))
        .collect())
}

pub(super) fn optional_int_values(
    batch: &RecordBatch,
    column: &str,
    stage: &str,
) -> Result<Vec<Option<i64>>, RepoIntelligenceError> {
    let Some(array) = batch.column_by_name(column) else {
        return Ok(vec![None; batch.num_rows()]);
    };
    if matches!(array.data_type(), DataType::Null) {
        return Ok(vec![None; array.len()]);
    }
    if let Some(values) = array.as_any().downcast_ref::<Int32Array>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| i64::from(values.value(index))))
            .collect());
    }
    if let Some(values) = array.as_any().downcast_ref::<Int64Array>() {
        return Ok((0..values.len())
            .map(|index| (!values.is_null(index)).then(|| values.value(index)))
            .collect());
    }
    Err(ast_query_contract_error(
        stage,
        format!(
            "column `{column}` expected Int32 or Int64 values but found {:?}",
            array.data_type()
        ),
    ))
}

pub(super) fn ast_query_request_error(message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to build Modelica ast-query request batch: {}",
            message.into()
        ),
    }
}

pub(super) fn ast_query_contract_error(
    stage: &str,
    message: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "invalid Modelica ast-query {stage} contract: {}",
            message.into()
        ),
    }
}
