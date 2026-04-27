#[cfg(feature = "duckdb")]
mod arrow_snapshot;
mod duckdb;
mod schema;
mod snapshot;
mod storage;
mod types;

pub(super) use duckdb::{
    default_local_duckdb_cache_path, load_cached_index_from_duckdb, save_cached_index_to_duckdb,
};
pub(super) use schema::{
    LINK_GRAPH_CACHE_SCHEMA_VERSION, cache_schema_fingerprint, cache_slot_key,
};
pub(super) use storage::{load_cached_index_from_valkey, save_cached_index_to_valkey};
pub(in crate::link_graph::index::build) use types::CacheLookupOutcome;
