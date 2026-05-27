use std::{collections::HashMap, sync::Arc};

use arrow::array::{ArrayRef, StringArray};
use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

const DOCUMENT_RESOURCE_TABLE: &str = "studio_document_extract_resource_cache";
const DOCUMENT_EXTRACT_STATUS_TABLE: &str = "studio_document_extract_status_cache";

pub(super) fn document_resource_schema() -> SchemaRef {
    arrow_schema_ref(&document_resource_contract())
}

pub(super) fn document_extract_status_schema() -> SchemaRef {
    arrow_schema_ref(&document_extract_status_contract())
}

pub(super) fn validate_document_resource_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_batch_schema(
        batch,
        &document_resource_contract(),
        "document resource cache schema",
    )
}

pub(super) fn validate_document_extract_status_batch(batch: &RecordBatch) -> Result<(), String> {
    validate_batch_schema(
        batch,
        &document_extract_status_contract(),
        "document extract status cache schema",
    )
}

fn document_resource_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        DOCUMENT_RESOURCE_TABLE,
        true,
        vec![
            nullable_utf8_column("sourcePath"),
            nullable_utf8_column("resourceType"),
            nullable_utf8_column("resourcePath"),
            nullable_int32_column("pageIndex"),
            nullable_utf8_column("caption"),
            nullable_utf8_column("content"),
            nullable_utf8_column("mimeType"),
            nullable_utf8_column("status"),
            nullable_utf8_column("elementId"),
        ],
    )
}

fn document_extract_status_contract() -> ArrowSchemaContract {
    ArrowSchemaContract::new(
        DOCUMENT_EXTRACT_STATUS_TABLE,
        true,
        vec![
            nullable_utf8_column("jobId"),
            nullable_utf8_column("sourcePath"),
            nullable_utf8_column("outputDir"),
            nullable_utf8_column("contentHash"),
            nullable_utf8_column("status"),
            nullable_int32_column("attemptCount"),
            nullable_int64_column("createdAtMs"),
            nullable_int64_column("startedAtMs"),
            nullable_int64_column("finishedAtMs"),
            nullable_utf8_column("errorMessage"),
        ],
    )
}

fn arrow_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
    let mut metadata = HashMap::new();
    metadata.insert(
        WENDAO_TABLE_METADATA_KEY.to_string(),
        contract.table_name().to_string(),
    );
    Arc::new(build_arrow_schema(contract, metadata))
}

fn validate_batch_schema(
    batch: &RecordBatch,
    contract: &ArrowSchemaContract,
    context: &str,
) -> Result<(), String> {
    validate_record_batch_schema_with_options(
        batch,
        contract,
        ArrowSchemaValidationOptions::new()
            .with_nullability_policy(ArrowSchemaNullabilityPolicy::Exact),
    )
    .map_err(|error| format!("validate Studio {context}: {error}"))
}

fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

fn nullable_int32_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int32)
}

fn nullable_int64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Int64)
}

pub(super) fn string_column<'a>(values: impl IntoIterator<Item = &'a str>) -> ArrayRef {
    Arc::new(StringArray::from_iter_values(values)) as ArrayRef
}

#[cfg(test)]
#[path = "../../../../../../../tests/unit/gateway/studio/router/handlers/analysis/document_extract/arrow_cache/schema.rs"]
mod tests;
