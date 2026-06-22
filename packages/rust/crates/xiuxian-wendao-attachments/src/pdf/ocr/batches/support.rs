//! Shared OCR Arrow schema and column helpers.

use std::{collections::HashMap, sync::Arc};

use arrow::array::{Array, ArrayRef, BooleanArray, Float64Array, Int32Array, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

use crate::pdf::ocr::types::{PdfOcrShardInput, PdfOcrShardResult};

const OCR_SHARD_INPUT_TABLE: &str = "pdf_ocr_shard_input";
const OCR_SHARD_RESULT_TABLE: &str = "pdf_ocr_shard_result";
const OCR_RESULT_RESOURCE_TABLE: &str = "pdf_ocr_result_resource";

pub(super) fn input_string_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> String,
{
    Arc::new(StringArray::from(
        inputs.iter().map(value).collect::<Vec<_>>(),
    ))
}

pub(super) fn input_int_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> u32,
{
    Arc::new(Int32Array::from(
        inputs
            .iter()
            .map(|input| i32::try_from(value(input)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

pub(super) fn input_float_column<F>(inputs: &[PdfOcrShardInput], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardInput) -> f64,
{
    Arc::new(Float64Array::from(
        inputs.iter().map(value).collect::<Vec<_>>(),
    ))
}

pub(super) fn result_string_column<F>(results: &[PdfOcrShardResult], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardResult) -> String,
{
    Arc::new(StringArray::from(
        results.iter().map(value).collect::<Vec<_>>(),
    ))
}

pub(super) fn result_int_column<F>(results: &[PdfOcrShardResult], value: F) -> ArrayRef
where
    F: Fn(&PdfOcrShardResult) -> u32,
{
    Arc::new(Int32Array::from(
        results
            .iter()
            .map(|result| i32::try_from(value(result)).unwrap_or(i32::MAX))
            .collect::<Vec<_>>(),
    ))
}

pub(super) fn validate_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    label: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(batch, contract, exact_schema_options())
        .map_err(|error| format!("{label} schema validation: {error}"))
}

pub(super) fn string_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a StringArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Utf8"))
}

pub(super) fn int32_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a Int32Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Int32"))
}

pub(super) fn float64_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a Float64Array, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<Float64Array>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Float64"))
}

pub(super) fn bool_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
) -> Result<&'a BooleanArray, String> {
    batch
        .column_by_name(name)
        .ok_or_else(|| format!("missing OCR shard result `{name}` column"))?
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| format!("OCR shard result `{name}` column is not Boolean"))
}

pub(super) fn required_string(
    column: &StringArray,
    row: usize,
    name: &str,
) -> Result<String, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row).to_string())
}

pub(super) fn required_bool(column: &BooleanArray, row: usize, name: &str) -> Result<bool, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row))
}

pub(super) fn optional_string(column: &StringArray, row: usize) -> Option<String> {
    (!column.is_null(row)).then(|| column.value(row).to_string())
}

pub(super) fn required_u32(column: &Int32Array, row: usize, name: &str) -> Result<u32, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    u32::try_from(column.value(row)).map_err(|_| {
        format!(
            "OCR shard result `{name}` must be non-negative at row {row}: {}",
            column.value(row)
        )
    })
}

pub(super) fn required_u16(column: &Int32Array, row: usize, name: &str) -> Result<u16, String> {
    let value = required_u32(column, row, name)?;
    u16::try_from(value)
        .map_err(|_| format!("OCR shard result `{name}` must fit into u16 at row {row}: {value}"))
}

pub(super) fn required_f64(column: &Float64Array, row: usize, name: &str) -> Result<f64, String> {
    if column.is_null(row) {
        return Err(format!("OCR shard result `{name}` is null at row {row}"));
    }
    Ok(column.value(row))
}

pub(super) fn optional_f64(column: &Float64Array, row: usize) -> Option<f64> {
    (!column.is_null(row)).then(|| column.value(row))
}

pub(super) fn ocr_shard_input_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_SHARD_INPUT_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("sourcePath"),
            utf8_contract_column("sourceContentHash"),
            int32_contract_column("pageIndex"),
            utf8_contract_column("imagePath"),
            utf8_contract_column("imageMimeType"),
            utf8_contract_column("rasterSha256"),
            utf8_contract_column("renderProfile"),
            utf8_contract_column("ocrProfile"),
            utf8_contract_column("ocrEngine"),
            utf8_contract_column("preferredLanguages"),
            float64_contract_column("minConfidence"),
            bool_contract_column("preserveLayout"),
            int32_contract_column("rasterWidthPx"),
            int32_contract_column("rasterHeightPx"),
            int32_contract_column("renderDpi"),
            int32_contract_column("rotationDegrees"),
            float64_contract_column("cropLeft"),
            float64_contract_column("cropBottom"),
            float64_contract_column("cropRight"),
            float64_contract_column("cropTop"),
            float64_contract_column("pointToPixelScaleX"),
            float64_contract_column("pointToPixelScaleY"),
            utf8_contract_column("shardElementId"),
            utf8_contract_column("shardType"),
            int32_contract_column("regionIndex"),
            utf8_contract_column("parentShardElementId"),
            utf8_contract_column("readingOrderKey"),
            int32_contract_column("sourcePagePixelLeft"),
            int32_contract_column("sourcePagePixelTop"),
            int32_contract_column("sourcePagePixelRight"),
            int32_contract_column("sourcePagePixelBottom"),
        ],
    )
}

pub(super) fn ocr_shard_result_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_SHARD_RESULT_TABLE,
        true,
        vec![
            utf8_contract_column("contractVersion"),
            utf8_contract_column("sourcePath"),
            utf8_contract_column("sourceContentHash"),
            int32_contract_column("pageIndex"),
            utf8_contract_column("imagePath"),
            utf8_contract_column("imageMimeType"),
            utf8_contract_column("rasterSha256"),
            utf8_contract_column("renderProfile"),
            utf8_contract_column("ocrProfile"),
            utf8_contract_column("status"),
            nullable_utf8_contract_column("text"),
            utf8_contract_column("textMimeType"),
            nullable_float64_contract_column("confidence"),
            nullable_utf8_contract_column("errorMessage"),
            utf8_contract_column("shardElementId"),
            utf8_contract_column("elementId"),
        ],
    )
}

pub(super) fn document_resource_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        OCR_RESULT_RESOURCE_TABLE,
        true,
        vec![
            nullable_utf8_contract_column("sourcePath"),
            nullable_utf8_contract_column("resourceType"),
            nullable_utf8_contract_column("resourcePath"),
            nullable_int32_contract_column("pageIndex"),
            nullable_utf8_contract_column("caption"),
            nullable_utf8_contract_column("content"),
            nullable_utf8_contract_column("mimeType"),
            nullable_utf8_contract_column("status"),
            nullable_utf8_contract_column("elementId"),
        ],
    )
}

pub(super) fn record_batch(
    contract: &ArrowSchemaContract,
    columns: Vec<ArrayRef>,
    context: &'static str,
) -> Result<RecordBatch, String> {
    let batch = RecordBatch::try_new(schema_ref(contract), columns)
        .map_err(|error| format!("{context}: {error}"))?;
    validate_batch_schema(&batch, contract, context)?;
    Ok(batch)
}

fn schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    Arc::new(build_arrow_schema(
        contract,
        [(
            WENDAO_TABLE_METADATA_KEY.to_string(),
            contract.table_name().to_string(),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    ))
}

const fn exact_schema_options() -> ArrowSchemaValidationOptions {
    ArrowSchemaValidationOptions::new().with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact)
}

const fn utf8_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

const fn nullable_utf8_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

const fn int32_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Int32)
}

const fn nullable_int32_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int32)
}

const fn float64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Float64)
}

const fn nullable_float64_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Float64)
}

const fn bool_contract_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
}
