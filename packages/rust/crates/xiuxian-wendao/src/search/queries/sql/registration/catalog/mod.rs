//! `search::queries::sql::registration::catalog` owns Wendao sql registration catalog behavior.

use std::collections::HashMap;
use std::sync::Arc;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use xiuxian_db_store::{
    ArrowSchemaColumn, ArrowSchemaContract, ArrowSchemaDataType, ArrowSchemaNullabilityPolicy,
    ArrowSchemaValidationOptions, WENDAO_TABLE_METADATA_KEY, build_arrow_schema,
    validate_record_batch_schema_with_options,
};

mod columns;
mod tables;
mod view_sources;

#[cfg(feature = "duckdb")]
pub(crate) use columns::build_columns_catalog_batch;
pub(crate) use columns::columns_catalog_schema;
#[cfg(not(feature = "duckdb"))]
pub(crate) use columns::register_columns_catalog_table;
#[cfg(feature = "duckdb")]
pub(crate) use tables::build_tables_catalog_batch;
#[cfg(not(feature = "duckdb"))]
pub(crate) use tables::register_tables_catalog_table;
pub(crate) use tables::tables_catalog_schema;
#[cfg(feature = "duckdb")]
pub(crate) use view_sources::build_view_sources_catalog_batch;
#[cfg(not(feature = "duckdb"))]
pub(crate) use view_sources::register_view_sources_catalog_table;
pub(crate) use view_sources::view_sources_catalog_schema;

fn catalog_schema_ref(contract: &ArrowSchemaContract) -> SchemaRef {
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

fn validate_catalog_batch(
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

const fn utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Utf8)
}

const fn nullable_utf8_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::nullable(name, ArrowSchemaDataType::Utf8)
}

const fn boolean_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::Boolean)
}

const fn uint64_column(name: &'static str) -> ArrowSchemaColumn {
    ArrowSchemaColumn::new(name, ArrowSchemaDataType::UInt64)
}
