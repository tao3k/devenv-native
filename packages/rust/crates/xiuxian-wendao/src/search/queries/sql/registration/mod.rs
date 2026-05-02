#[path = "catalog/mod.rs"]
mod catalog;
#[cfg(not(feature = "duckdb"))]
mod collector;
mod local;
mod naming;
mod repo;
#[cfg(not(feature = "duckdb"))]
mod session;
mod surface;
mod table;
#[path = "views/mod.rs"]
mod views;

#[cfg(feature = "duckdb")]
pub(crate) use catalog::{
    build_columns_catalog_batch, build_tables_catalog_batch, build_view_sources_catalog_batch,
};
#[cfg(not(feature = "duckdb"))]
pub(crate) use collector::register_datafusion_sql_query_surface;
pub(crate) use surface::build_sql_query_surface;
#[cfg(feature = "duckdb")]
pub(crate) use surface::{SqlSurfaceAssembly, build_sql_surface_assembly};
pub(crate) use table::{
    RegisteredSqlColumn, RegisteredSqlTable, RegisteredSqlViewSource,
    STUDIO_SQL_CATALOG_TABLE_NAME, STUDIO_SQL_COLUMNS_CATALOG_TABLE_NAME,
    STUDIO_SQL_VIEW_SOURCES_CATALOG_TABLE_NAME, SqlQuerySurface,
};
#[cfg(feature = "duckdb")]
pub(crate) use views::{collect_local_logical_view_sql, collect_repo_logical_view_sqls};
