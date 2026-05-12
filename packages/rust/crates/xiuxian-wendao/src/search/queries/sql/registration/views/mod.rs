//! `search::queries::sql::registration::views` owns Wendao sql registration views behavior.

mod local;
mod repo;

#[cfg(feature = "duckdb")]
pub(crate) use local::collect_local_logical_view_sql;
pub(crate) use local::collect_local_logical_views;
#[cfg(not(feature = "duckdb"))]
pub(crate) use local::register_local_logical_views;
#[cfg(feature = "duckdb")]
pub(crate) use repo::collect_repo_logical_view_sqls;
pub(crate) use repo::collect_repo_logical_views;
#[cfg(not(feature = "duckdb"))]
pub(crate) use repo::register_repo_logical_views;
