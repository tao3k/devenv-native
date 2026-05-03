/// Shared bounded-work markdown SQL surface for workdir-local retrieval.
#[path = "bounded_work_markdown/mod.rs"]
pub mod bounded_work_markdown;
#[path = "execution/mod.rs"]
pub(crate) mod execution;
#[cfg(feature = "runtime-transport")]
/// Runtime transport provider for SQL Flight route integration.
#[path = "provider/mod.rs"]
pub mod provider;
#[cfg(feature = "search-runtime")]
#[path = "registration/mod.rs"]
pub(crate) mod registration;

#[cfg(feature = "search-runtime")]
pub use self::execution::query_sql_payload;
pub use self::execution::{SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload};
#[cfg(feature = "search-runtime")]
pub(crate) use self::execution::{
    configured_parquet_query_engine, engine_batches_rows_payload, execute_sql_query,
    try_execute_published_parquet_query,
};
#[cfg(feature = "search-runtime")]
pub(crate) use self::registration::SqlQuerySurface;

#[cfg(all(test, feature = "search-runtime"))]
#[path = "../../../../tests/unit/search/queries/sql/mod.rs"]
mod tests;
