use std::sync::Arc;

use arrow::array::{StringArray, UInt64Array};
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
#[cfg(not(feature = "duckdb"))]
use datafusion::datasource::MemTable;
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use crate::search::queries::sql::registration::{
    RegisteredSqlViewSource, STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
};

use super::{
    catalog_schema_ref, nullable_utf8_column, uint64_column, utf8_column, validate_catalog_batch,
};

pub(crate) fn view_sources_catalog_schema() -> Arc<Schema> {
    catalog_schema_ref(&view_sources_catalog_contract())
}

pub(crate) fn build_view_sources_catalog_batch(
    view_sources: &[RegisteredSqlViewSource],
) -> Result<RecordBatch, String> {
    let schema = view_sources_catalog_schema();
    let batch = RecordBatch::try_new(
        Arc::clone(&schema),
        vec![
            Arc::new(StringArray::from(
                view_sources
                    .iter()
                    .map(|view_source| Some(view_source.sql_view_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                view_sources
                    .iter()
                    .map(|view_source| Some(view_source.source_sql_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                view_sources
                    .iter()
                    .map(|view_source| Some(view_source.source_engine_table_name.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                view_sources
                    .iter()
                    .map(|view_source| Some(view_source.corpus.as_str()))
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                view_sources
                    .iter()
                    .map(|view_source| view_source.repo_id.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(UInt64Array::from(
                view_sources
                    .iter()
                    .map(|view_source| Some(view_source.source_ordinal))
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL view-source catalog batch: {error}")
    })?;
    validate_catalog_batch(
        &batch,
        &view_sources_catalog_contract(),
        "studio SQL Flight provider built invalid SQL view-source catalog batch",
    )?;
    Ok(batch)
}

#[cfg(not(feature = "duckdb"))]
pub(crate) fn register_view_sources_catalog_table(
    query_engine: &SearchEngineContext,
    view_sources: &[RegisteredSqlViewSource],
) -> Result<(), String> {
    let schema = view_sources_catalog_schema();
    let batch = build_view_sources_catalog_batch(view_sources)?;
    let mem_table = MemTable::try_new(schema, vec![vec![batch]]).map_err(|error| {
        format!("studio SQL Flight provider failed to build SQL view-source catalog: {error}")
    })?;
    query_engine
        .session()
        .deregister_table(STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME)
        .map_err(|error| {
            format!("studio SQL Flight provider failed to reset SQL view-source catalog: {error}")
        })?;
    query_engine
        .session()
        .register_table(
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
            Arc::new(mem_table),
        )
        .map_err(|error| {
            format!(
                "studio SQL Flight provider failed to register SQL view-source catalog: {error}"
            )
        })?;
    Ok(())
}

fn view_sources_catalog_contract() -> xiuxian_db_store::ArrowSchemaContract {
    xiuxian_db_store::ArrowSchemaContract::new(
        STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
        true,
        vec![
            utf8_column("sql_view_name"),
            utf8_column("source_sql_table_name"),
            utf8_column("source_engine_table_name"),
            utf8_column("corpus"),
            nullable_utf8_column("repo_id"),
            uint64_column("source_ordinal"),
        ],
    )
}
