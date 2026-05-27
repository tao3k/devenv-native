use std::sync::Arc;

use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
#[cfg(not(feature = "duckdb"))]
use datafusion::datasource::MemTable;
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use crate::search::queries::sql::registration::{
    RegisteredSqlColumn, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
};

use super::{
    boolean_column, catalog_schema_ref, nullable_utf8_column, uint64_column, utf8_column,
    validate_catalog_batch,
};

pub(crate) fn columns_catalog_schema() -> Arc<Schema> {
    catalog_schema_ref(&columns_catalog_contract())
}

pub(crate) fn build_columns_catalog_batch(
    columns: &[RegisteredSqlColumn],
) -> Result<RecordBatch, String> {
    let schema = columns_catalog_schema();
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.sql_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.engine_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.column_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| column.source_column_name.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.data_type.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.is_nullable))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                columns
                    .iter()
                    .map(|column| Some(column.ordinal_position))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.corpus.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.scope.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.sql_object_kind.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| Some(column.column_origin_kind.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                columns
                    .iter()
                    .map(|column| column.repo_id.as_deref())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL column catalog batch: {error}")
    })?;
    validate_catalog_batch(
        &batch,
        &columns_catalog_contract(),
        "studio SQL Flight provider built invalid SQL column catalog batch",
    )?;
    Ok(batch)
}

#[cfg(not(feature = "duckdb"))]
pub(crate) fn register_columns_catalog_table(
    query_engine: &SearchEngineContext,
    columns: &[RegisteredSqlColumn],
) -> Result<(), String> {
    let schema = columns_catalog_schema();
    let batch = build_columns_catalog_batch(columns)?;
    let mem_table = MemTable::try_new(schema, vec![vec![batch]]).map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL column catalog: {error}")
    })?;
    query_engine
        .session()
        .deregister_table(STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME)
        .map_err(|error| {
            format!("studio SQL Flight provider failed to reset SQL column catalog: {error}")
        })?;
    query_engine
        .session()
        .register_table(STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME, Arc::new(mem_table))
        .map_err(|error| {
            format!("studio SQL Flight provider failed to register SQL column catalog: {error}")
        })?;
    Ok(())
}

fn columns_catalog_contract() -> xiuxian_db_store::ArrowSchemaContract {
    xiuxian_db_store::ArrowSchemaContract::new(
        STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
        true,
        vec![
            utf8_column("sql_table_name"),
            utf8_column("engine_table_name"),
            utf8_column("column_name"),
            nullable_utf8_column("source_column_name"),
            utf8_column("data_type"),
            boolean_column("is_nullable"),
            uint64_column("ordinal_position"),
            utf8_column("corpus"),
            utf8_column("scope"),
            utf8_column("sql_object_kind"),
            utf8_column("column_origin_kind"),
            nullable_utf8_column("repo_id"),
        ],
    )
}
