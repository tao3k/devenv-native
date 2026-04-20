use std::sync::Arc;

use arrow::array::ArrayRef;
use arrow::compute::cast;
use arrow::datatypes::{DataType, Field, Schema};
use xiuxian_db_store::EngineRecordBatch;

use crate::duckdb::{LocalRelationEngineKind, ParquetQueryEngine};
use crate::search::queries::SearchQueryService;
use crate::search::queries::sql::execution::service::SqlQueryExecutionRoute;
use crate::search::queries::sql::execution::service::execute_sql_query_with_route;
use crate::search::queries::sql::try_execute_published_parquet_query;
use crate::search::{SearchCorpusKind, SearchPlaneService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FlightSqlStatementRoute {
    SharedSql {
        engine_kind: LocalRelationEngineKind,
    },
    LocalParquet {
        corpus: SearchCorpusKind,
        table_name: String,
        engine_kind: LocalRelationEngineKind,
    },
}

pub(super) struct FlightSqlStatementExecution {
    pub(super) route: FlightSqlStatementRoute,
    pub(super) batches: Vec<EngineRecordBatch>,
}

pub(super) async fn execute_flightsql_statement_query(
    service: &SearchQueryService,
    query_engine: Option<&ParquetQueryEngine>,
    query_text: &str,
) -> Result<FlightSqlStatementExecution, String> {
    if let Some(result) = try_execute_published_parquet_statement_query(
        service.search_plane(),
        query_engine,
        query_text,
    )
    .await?
    {
        return Ok(result);
    }

    let (route, result) = execute_sql_query_with_route(service, query_text).await?;
    let (_metadata, batches) = result.into_parts();
    Ok(FlightSqlStatementExecution {
        route: match route {
            SqlQueryExecutionRoute::SharedSql { engine_kind } => {
                FlightSqlStatementRoute::SharedSql { engine_kind }
            }
            SqlQueryExecutionRoute::LocalParquet {
                corpus,
                table_name,
                engine_kind,
            } => FlightSqlStatementRoute::LocalParquet {
                corpus,
                table_name,
                engine_kind,
            },
        },
        batches,
    })
}

async fn try_execute_published_parquet_statement_query(
    service: &SearchPlaneService,
    query_engine: Option<&ParquetQueryEngine>,
    query_text: &str,
) -> Result<Option<FlightSqlStatementExecution>, String> {
    let Some(result) =
        try_execute_published_parquet_query(service, query_engine, query_text).await?
    else {
        return Ok(None);
    };
    let batches = normalize_flightsql_statement_batches(result.batches)?;
    Ok(Some(FlightSqlStatementExecution {
        route: FlightSqlStatementRoute::LocalParquet {
            corpus: result.corpus,
            table_name: result.table_name,
            engine_kind: result.engine_kind,
        },
        batches,
    }))
}

fn normalize_flightsql_statement_batches(
    batches: Vec<EngineRecordBatch>,
) -> Result<Vec<EngineRecordBatch>, String> {
    batches
        .into_iter()
        .map(normalize_flightsql_statement_batch)
        .collect()
}

fn normalize_flightsql_statement_batch(
    batch: EngineRecordBatch,
) -> Result<EngineRecordBatch, String> {
    let schema = batch.schema();
    let mut changed = false;
    let mut fields = Vec::with_capacity(schema.fields().len());
    let mut columns = Vec::with_capacity(batch.num_columns());

    for (field, column) in schema.fields().iter().zip(batch.columns().iter()) {
        let (normalized_field, normalized_column, column_changed) =
            normalize_flightsql_statement_column(field.as_ref(), column.clone())?;
        fields.push(normalized_field);
        columns.push(normalized_column);
        changed |= column_changed;
    }

    if !changed {
        return Ok(batch);
    }

    EngineRecordBatch::try_new(
        Arc::new(Schema::new_with_metadata(fields, schema.metadata().clone())),
        columns,
    )
    .map_err(|error| format!("FlightSQL failed to rebuild normalized statement batch: {error}"))
}

fn normalize_flightsql_statement_column(
    field: &Field,
    column: ArrayRef,
) -> Result<(Field, ArrayRef, bool), String> {
    match field.data_type() {
        DataType::Utf8 | DataType::LargeUtf8 => {
            let normalized_column =
                cast(column.as_ref(), &DataType::Utf8View).map_err(|error| {
                    format!(
                        "FlightSQL failed to normalize string column `{}` to Utf8View: {error}",
                        field.name()
                    )
                })?;
            Ok((
                Field::new(field.name(), DataType::Utf8View, field.is_nullable())
                    .with_metadata(field.metadata().clone()),
                normalized_column,
                true,
            ))
        }
        _ => Ok((field.clone(), column, false)),
    }
}
