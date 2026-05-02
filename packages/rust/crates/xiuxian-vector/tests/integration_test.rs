//! Cargo entry point for xiuxian-vector integration tests.
#![cfg(feature = "vector-store")]

#[path = "integration/columnar_tables.rs"]
mod columnar_tables;
#[path = "integration/data_layer_snapshots.rs"]
mod data_layer_snapshots;
#[path = "integration/maintenance.rs"]
mod maintenance;
#[path = "integration/merge_insert.rs"]
mod merge_insert;
#[path = "integration/migration.rs"]
mod migration;
#[path = "integration/observability.rs"]
mod observability;
#[path = "integration/partitioning.rs"]
mod partitioning;
#[path = "integration/path_handling.rs"]
mod path_handling;
#[path = "integration/scalar_index.rs"]
mod scalar_index;
#[path = "integration/schema_encoding.rs"]
mod schema_encoding;
#[path = "integration/search.rs"]
mod search;
#[path = "integration/search_cache.rs"]
mod search_cache;
#[path = "integration/search_engine.rs"]
mod search_engine;
#[path = "integration/snapshot_support.rs"]
mod snapshot_support;
#[path = "integration/store.rs"]
mod store;
#[path = "integration/vector_index.rs"]
mod vector_index;
