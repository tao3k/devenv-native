//! `search::queries::sql::registration::catalog` owns Wendao sql registration catalog behavior.

mod batch_helpers;
mod column_helpers;
mod columns;
mod tables;
mod view_sources;

pub(super) use batch_helpers::{catalog_schema_ref, validate_catalog_batch};
pub(super) use column_helpers::{boolean_column, nullable_utf8_column, uint64_column, utf8_column};

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
