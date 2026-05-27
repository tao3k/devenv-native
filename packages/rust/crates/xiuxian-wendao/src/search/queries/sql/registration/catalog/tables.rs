use std::sync::Arc;

use arrow::array::{StringArray, UInt64Array};
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
#[cfg(not(feature = "duckdb"))]
use datafusion::datasource::MemTable;
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use crate::search::queries::sql::registration::{
    RegisteredSqlTable, STUDIO_SQL_CATALOG_TABLE_NAME,
};

use super::{
    catalog_schema_ref, nullable_utf8_column, uint64_column, utf8_column, validate_catalog_batch,
};

pub(crate) fn tables_catalog_schema() -> Arc<Schema> {
    catalog_schema_ref(&tables_catalog_contract())
}

pub(crate) fn build_tables_catalog_batch(
    tables: &[RegisteredSqlTable],
) -> Result<RecordBatch, String> {
    let schema = tables_catalog_schema();
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| Some(table.sql_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| Some(table.engine_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| Some(table.corpus.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| Some(table.scope.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| Some(table.sql_object_kind.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                tables
                    .iter()
                    .map(|table| Some(table.source_count))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                tables
                    .iter()
                    .map(|table| table.repo_id.as_deref())
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL table catalog batch: {error}")
    })?;
    validate_catalog_batch(
        &batch,
        &tables_catalog_contract(),
        "studio SQL Flight provider built invalid SQL table catalog batch",
    )?;
    Ok(batch)
}

#[cfg(not(feature = "duckdb"))]
pub(crate) fn register_tables_catalog_table(
    query_engine: &SearchEngineContext,
    tables: &[RegisteredSqlTable],
) -> Result<(), String> {
    let schema = tables_catalog_schema();
    let batch = build_tables_catalog_batch(tables)?;
    let mem_table = MemTable::try_new(schema, vec![vec![batch]]).map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL table catalog: {error}")
    })?;
    query_engine
        .session()
        .deregister_table(STUDIO_SQL_CATALOG_TABLE_NAME)
        .map_err(|error| {
            format!("studio SQL Flight provider failed to reset SQL table catalog: {error}")
        })?;
    query_engine
        .session()
        .register_table(STUDIO_SQL_CATALOG_TABLE_NAME, Arc::new(mem_table))
        .map_err(|error| {
            format!("studio SQL Flight provider failed to register SQL table catalog: {error}")
        })?;
    Ok(())
}

fn tables_catalog_contract() -> xiuxian_db_store::ArrowSchemaContract {
    xiuxian_db_store::ArrowSchemaContract::new(
        STUDIO_SQL_CATALOG_TABLE_NAME,
        true,
        vec![
            utf8_column("sql_table_name"),
            utf8_column("engine_table_name"),
            utf8_column("corpus"),
            utf8_column("scope"),
            utf8_column("sql_object_kind"),
            uint64_column("source_count"),
            nullable_utf8_column("repo_id"),
        ],
    )
}
