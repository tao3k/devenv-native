use crate::duckdb::LocalRelationEngineKind;
use crate::search::SearchCorpusKind;
use crate::search::queries::SearchQueryService;

use super::parquet::try_execute_published_parquet_query;
use super::result::{SqlQueryMetadata, SqlQueryPayload, SqlQueryResult};
use super::shared::execute_shared_sql_query;
use crate::search::queries::sql::registration::SqlQuerySurface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SqlQueryExecutionRoute {
    SharedSql {
        engine_kind: LocalRelationEngineKind,
    },
    LocalParquet {
        corpus: SearchCorpusKind,
        table_name: String,
        engine_kind: LocalRelationEngineKind,
    },
}

pub(crate) async fn execute_sql_query(
    service: &SearchQueryService,
    query_text: &str,
) -> Result<SqlQueryResult, String> {
    let (_route, result) = execute_sql_query_internal(service, query_text).await?;
    Ok(result)
}

pub(crate) async fn execute_sql_query_with_route(
    service: &SearchQueryService,
    query_text: &str,
) -> Result<(SqlQueryExecutionRoute, SqlQueryResult), String> {
    execute_sql_query_internal(service, query_text).await
}

async fn execute_sql_query_internal(
    service: &SearchQueryService,
    query_text: &str,
) -> Result<(SqlQueryExecutionRoute, SqlQueryResult), String> {
    if let Some(routed) =
        try_execute_published_parquet_query(service.search_plane(), None, query_text).await?
    {
        let query_surface = service.open_sql_surface().await?;
        let metadata =
            sql_query_metadata_with_result_counts(&query_surface, routed.batches.as_slice());
        return Ok((
            SqlQueryExecutionRoute::LocalParquet {
                corpus: routed.corpus,
                table_name: routed.table_name,
                engine_kind: routed.engine_kind,
            },
            SqlQueryResult::new(metadata, routed.batches),
        ));
    }

    let (engine_kind, query_surface, engine_batches) =
        execute_shared_sql_query(service, query_text).await?;
    let metadata = sql_query_metadata_with_result_counts(&query_surface, engine_batches.as_slice());
    Ok((
        SqlQueryExecutionRoute::SharedSql { engine_kind },
        SqlQueryResult::new(metadata, engine_batches),
    ))
}

fn sql_query_metadata_with_result_counts(
    query_surface: &SqlQuerySurface,
    engine_batches: &[xiuxian_db_store::EngineRecordBatch],
) -> SqlQueryMetadata {
    let mut metadata = sql_query_metadata(query_surface);
    metadata.result_row_count = engine_batches
        .iter()
        .map(xiuxian_db_store::EngineRecordBatch::num_rows)
        .sum();
    metadata.result_batch_count = engine_batches.len();
    metadata
}

fn sql_query_metadata(query_surface: &SqlQuerySurface) -> SqlQueryMetadata {
    SqlQueryMetadata {
        catalog_table_name: query_surface.catalog_table_name.clone(),
        column_catalog_table_name: query_surface.column_catalog_table_name.clone(),
        view_source_catalog_table_name: query_surface.view_source_catalog_table_name.clone(),
        supports_information_schema: true,
        registered_tables: query_surface.registered_table_names(),
        registered_table_count: query_surface.registered_table_count(),
        registered_view_count: query_surface.registered_view_count(),
        registered_column_count: query_surface.registered_column_count(),
        registered_view_source_count: query_surface.registered_view_source_count(),
        result_batch_count: 0,
        result_row_count: 0,
        registered_input_bytes: None,
        result_bytes: None,
        local_relation_materialization_state: None,
        local_temp_storage_peak_bytes: None,
        local_relation_engine: None,
        duckdb_registration_strategy: None,
        registered_input_batch_count: None,
        registered_input_row_count: None,
        registration_time_ms: None,
        local_query_execution_time_ms: None,
    }
}

/// Execute one request-scoped SQL query and serialize the result payload.
///
/// # Errors
///
/// Returns an error when the request-scoped SQL surface cannot be registered
/// or when the SQL statement cannot be executed or serialized into the stable
/// CLI-friendly payload shape.
pub async fn query_sql_payload(
    service: &SearchQueryService,
    query_text: &str,
) -> Result<SqlQueryPayload, String> {
    execute_sql_query(service, query_text).await?.payload()
}
