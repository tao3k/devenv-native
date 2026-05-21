//! `search::queries::sql::execution` owns Wendao queries sql execution behavior.

#[cfg(feature = "search-runtime")]
#[path = "parquet.rs"]
mod parquet;
#[path = "result.rs"]
mod result;
#[cfg(feature = "search-runtime")]
#[path = "service.rs"]
pub(crate) mod service;
#[cfg(feature = "search-runtime")]
#[path = "shared.rs"]
mod shared;

#[cfg(feature = "search-runtime")]
pub(crate) use self::parquet::{
    configured_parquet_query_engine, try_execute_published_parquet_query,
};
#[cfg(feature = "search-runtime")]
pub(crate) use self::result::engine_batches_rows_payload;
pub use self::result::{SqlBatchPayload, SqlColumnPayload, SqlQueryMetadata, SqlQueryPayload};
#[cfg(feature = "search-runtime")]
pub(crate) use self::service::execute_sql_query;
#[cfg(feature = "search-runtime")]
pub use self::service::query_sql_payload;
