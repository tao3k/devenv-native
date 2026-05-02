//! DuckDB-local Arrow cache payload encoding for link-graph indices.

mod api;
mod decode;
mod encode;
mod ipc;
mod page_index;
mod primitive;

pub(in crate::link_graph::index::build::cache) use api::{
    LINK_GRAPH_DUCKDB_ARROW_CACHE_SCHEMA_VERSION, LinkGraphArrowSnapshotPayload,
    decode_arrow_cached_index_payload, duckdb_arrow_cache_schema_fingerprint,
    encode_arrow_cached_index_payload,
};
