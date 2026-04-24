#[cfg(feature = "duckdb")]
use std::collections::BTreeMap;

use xiuxian_db_store::EngineRecordBatch;

use crate::duckdb::LocalRelationEngineKind;
#[cfg(feature = "duckdb")]
use crate::duckdb::{
    DuckDbLocalRelationEngine, LocalRelationEngine, resolve_search_duckdb_runtime,
};
use crate::search::queries::SearchQueryService;
#[cfg(not(feature = "duckdb"))]
use crate::search::queries::sql::registration::SqlQuerySurface;
#[cfg(feature = "duckdb")]
use crate::search::queries::sql::registration::{
    RegisteredSqlTable, SqlQuerySurface, SqlSurfaceAssembly, build_columns_catalog_batch,
    build_sql_surface_assembly, build_tables_catalog_batch, build_view_sources_catalog_batch,
    collect_local_logical_view_sql, collect_repo_logical_view_sqls,
};

pub(crate) async fn execute_shared_sql_query(
    service: &SearchQueryService,
    query_text: &str,
) -> Result<
    (
        LocalRelationEngineKind,
        SqlQuerySurface,
        Vec<EngineRecordBatch>,
    ),
    String,
> {
    #[cfg(feature = "duckdb")]
    {
        let assembly = build_sql_surface_assembly(service.search_plane()).await?;
        let query_engine = configured_duckdb_shared_sql_engine()?;
        register_duckdb_shared_sql_surface(&query_engine, &assembly)?;
        let batches = query_engine
            .query_batches(query_text)
            .await
            .map_err(|error| {
                format!("shared SQL query execution failed for `{query_text}`: {error}")
            })?;
        Ok((LocalRelationEngineKind::DuckDb, assembly.surface, batches))
    }

    #[cfg(not(feature = "duckdb"))]
    {
        let query_core = service.open_datafusion_core().await?;
        let engine_batches = query_core
            .datafusion_engine()
            .sql_batches(query_text)
            .await
            .map_err(|error| {
                format!("shared SQL query execution failed for `{query_text}`: {error}")
            })?;
        Ok((
            LocalRelationEngineKind::DataFusion,
            query_core.surface().clone(),
            engine_batches,
        ))
    }
}

#[cfg(feature = "duckdb")]
fn configured_duckdb_shared_sql_engine() -> Result<DuckDbLocalRelationEngine, String> {
    let mut runtime = resolve_search_duckdb_runtime();
    runtime.enabled = true;
    DuckDbLocalRelationEngine::from_runtime(runtime)
}

#[cfg(feature = "duckdb")]
fn register_duckdb_shared_sql_surface(
    query_engine: &DuckDbLocalRelationEngine,
    assembly: &SqlSurfaceAssembly,
) -> Result<(), String> {
    let tables_by_name = assembly
        .surface
        .tables
        .iter()
        .cloned()
        .map(|table| (table.sql_table_name.clone(), table))
        .collect::<BTreeMap<String, RegisteredSqlTable>>();

    register_duckdb_parquet_tables(query_engine, &tables_by_name, &assembly.parquet_paths)?;
    register_duckdb_catalog_tables(query_engine, &assembly.surface)?;
    register_duckdb_logical_views(query_engine, &tables_by_name)?;
    Ok(())
}

#[cfg(feature = "duckdb")]
fn register_duckdb_parquet_tables(
    query_engine: &DuckDbLocalRelationEngine,
    tables_by_name: &BTreeMap<String, RegisteredSqlTable>,
    parquet_paths: &BTreeMap<String, std::path::PathBuf>,
) -> Result<(), String> {
    for table in tables_by_name
        .values()
        .filter(|table| table.sql_object_kind == "table")
    {
        let parquet_path = parquet_paths
            .get(table.sql_table_name.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "shared SQL surface should carry parquet path for `{}`",
                    table.sql_table_name
                )
            });
        query_engine.register_parquet_view(table.sql_table_name.as_str(), parquet_path)?;
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
fn register_duckdb_catalog_tables(
    query_engine: &DuckDbLocalRelationEngine,
    surface: &SqlQuerySurface,
) -> Result<(), String> {
    register_duckdb_catalog_batch(
        query_engine,
        surface.catalog_table_name.as_str(),
        build_tables_catalog_batch(surface.tables.as_slice())?,
    )?;
    register_duckdb_catalog_batch(
        query_engine,
        surface.column_catalog_table_name.as_str(),
        build_columns_catalog_batch(surface.columns.as_slice())?,
    )?;
    register_duckdb_catalog_batch(
        query_engine,
        surface.view_source_catalog_table_name.as_str(),
        build_view_sources_catalog_batch(surface.view_sources.as_slice())?,
    )?;
    Ok(())
}

#[cfg(feature = "duckdb")]
fn register_duckdb_catalog_batch(
    query_engine: &DuckDbLocalRelationEngine,
    table_name: &str,
    batch: EngineRecordBatch,
) -> Result<(), String> {
    let schema = batch.schema();
    query_engine.register_record_batches(table_name, schema, vec![batch])
}

#[cfg(feature = "duckdb")]
fn register_duckdb_logical_views(
    query_engine: &DuckDbLocalRelationEngine,
    tables_by_name: &BTreeMap<String, RegisteredSqlTable>,
) -> Result<(), String> {
    if let Some((_logical_view_name, view_sql)) = collect_local_logical_view_sql(tables_by_name) {
        query_engine.execute_batch_sql(view_sql.as_str())?;
    }
    for (_logical_view_name, view_sql) in collect_repo_logical_view_sqls(tables_by_name) {
        query_engine.execute_batch_sql(view_sql.as_str())?;
    }
    Ok(())
}
