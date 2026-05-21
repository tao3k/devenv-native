//! `link_graph::stats_cache` owns Wendao link graph stats cache behavior.

mod keys;
mod payload;
mod runtime;
mod schema;
mod types;
mod valkey;

pub use schema::LINK_GRAPH_STATS_CACHE_SCHEMA_VERSION;
pub use valkey::{valkey_stats_cache_del, valkey_stats_cache_get, valkey_stats_cache_set};
