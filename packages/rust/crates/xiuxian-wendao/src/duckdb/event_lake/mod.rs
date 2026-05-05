//! Wendao-owned event lake consumer helpers over `xiuxian-db-store` `DuckLake`.

mod config;
mod handle;
mod query;
mod record;
mod schema;
mod store;

pub use config::WendaoEventLakeLocalConfig;
pub use handle::WendaoEventLake;
pub use query::{
    WENDAO_EVENT_QUERY_DEFAULT_LIMIT, WENDAO_EVENT_QUERY_MAX_LIMIT, WendaoEventQuery,
    query_wendao_events,
};
pub use record::{WendaoEventRecord, WendaoEventTypeCount};
pub use schema::{
    WENDAO_EVENT_LAKE_EVENTS_TABLE, build_wendao_event_lake_table_sql, validate_wendao_event_batch,
    wendao_event_record_batch, wendao_event_schema,
};
pub use store::{
    WENDAO_EVENT_APPEND_DEFAULT_BATCH_ROWS, WENDAO_EVENT_LAKE_DEFAULT_ALIAS,
    WendaoEventLakeAppender, append_wendao_event_batches, append_wendao_events,
    ensure_wendao_event_lake_table, query_wendao_event_type_counts,
};
