#[cfg(feature = "search-runtime")]
#[path = "execution/parquet.rs"]
mod parquet;
#[path = "execution/result.rs"]
mod result;
#[cfg(feature = "search-runtime")]
#[path = "execution/service.rs"]
pub(crate) mod service;
#[cfg(feature = "search-runtime")]
#[path = "execution/shared.rs"]
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
