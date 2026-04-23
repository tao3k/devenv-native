use std::sync::Arc;

use arrow::array::{BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
#[cfg(not(feature = "duckdb"))]
use datafusion::datasource::MemTable;
#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

use crate::search::queries::sql::registration::RegisteredSqlColumn;
#[cfg(not(feature = "duckdb"))]
use crate::search::queries::sql::registration::STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME;

pub(crate) fn columns_catalog_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("sql_table_name", DataType::Utf8, false),
        Field::new("engine_table_name", DataType::Utf8, false),
        Field::new("column_name", DataType::Utf8, false),
        Field::new("source_column_name", DataType::Utf8, true),
        Field::new("data_type", DataType::Utf8, false),
        Field::new("is_nullable", DataType::Boolean, false),
        Field::new("ordinal_position", DataType::UInt64, false),
        Field::new("corpus", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("sql_object_kind", DataType::Utf8, false),
        Field::new("column_origin_kind", DataType::Utf8, false),
        Field::new("repo_id", DataType::Utf8, true),
    ]))
}

pub(crate) fn build_columns_catalog_batch(
    columns: &[RegisteredSqlColumn],
) -> Result<RecordBatch, String> {
    let schema = columns_catalog_schema();
    RecordBatch::try_new(
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
    })
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
