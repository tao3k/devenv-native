//! Record batch helpers for catalog registration.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaContract, ArrowSchemaNullabilityPolicy, ArrowSchemaValidationOptions,
    WENDAO_TABLE_METADATA_KEY, build_arrow_schema, validate_record_batch_schema_with_options,
};

pub(in crate::search::queries::sql::registration) fn catalog_schema_ref(
    contract: &ArrowSchemaContract,
) -> SchemaRef {
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

pub(in crate::search::queries::sql::registration) fn validate_catalog_batch(
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
    .map_err(|error| format!("{context}: {error}"))
}
