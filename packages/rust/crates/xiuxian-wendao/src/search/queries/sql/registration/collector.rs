use std::collections::BTreeMap;

use crate::search::SearchPlaneService;
use xiuxian_db_store::SearchEngineContext;

use super::catalog::{
    register_columns_catalog_table, register_tables_catalog_table,
    register_view_sources_catalog_table,
};
use super::{
    RegisteredSqlTable, STUDIO_SQL_CATALOG_TABLE_NAME, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
    STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME, SqlQuerySurface, local, repo, session, surface,
    views,
};

pub(crate) async fn register_datafusion_sql_query_surface(
    service: &SearchPlaneService,
) -> Result<(SearchEngineContext, SqlQuerySurface), String> {
    let assembled_surface = surface::build_sql_surface_assembly(service).await?;
    let datafusion_query_engine = session::new_datafusion_sql_query_engine();
    let SqlQuerySurface {
        catalog_table_name: _,
        column_catalog_table_name: _,
        view_source_catalog_table_name: _,
        tables,
        columns,
        view_sources,
    } = assembled_surface.surface;
    let tables_map = tables
        .iter()
        .cloned()
        .map(|table| (table.sql_table_name.clone(), table))
        .collect::<BTreeMap<String, RegisteredSqlTable>>();
    local::register_local_tables(
        &datafusion_query_engine,
        &tables_map,
        &assembled_surface.parquet_paths,
    )
    .await?;
    repo::register_repo_tables(
        &datafusion_query_engine,
        &tables_map,
        &assembled_surface.parquet_paths,
    )
    .await?;
    views::register_local_logical_views(&datafusion_query_engine, &tables_map).await?;
    views::register_repo_logical_views(&datafusion_query_engine, &tables_map).await?;
    register_tables_catalog_table(&datafusion_query_engine, tables.as_slice())?;
    register_view_sources_catalog_table(&datafusion_query_engine, view_sources.as_slice())?;
    register_columns_catalog_table(&datafusion_query_engine, columns.as_slice())?;
    Ok((
        datafusion_query_engine,
        SqlQuerySurface::new(
            STUDIO_SQL_CATALOG_TABLE_NAME,
            STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
            STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME,
            tables,
            columns,
            view_sources,
        ),
    ))
}
