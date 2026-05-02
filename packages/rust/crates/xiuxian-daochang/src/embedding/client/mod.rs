//! Embedding client trait and shared adapter implementation.

mod backend_dispatch;
mod batch;
mod chunk_dispatch;
mod init;
mod support;
mod types;

use types::{
    DEFAULT_EMBED_BATCH_MAX_CONCURRENCY, DEFAULT_EMBED_BATCH_MAX_SIZE,
    DEFAULT_EMBED_CACHE_MAX_ENTRIES, DEFAULT_EMBED_CACHE_TTL_SECS, EmbeddingDispatchRuntime,
    MAX_EMBED_BATCH_MAX_CONCURRENCY, MAX_EMBED_BATCH_MAX_SIZE, MAX_EMBED_CACHE_MAX_ENTRIES,
    MAX_EMBED_CACHE_TTL_SECS,
};
pub use types::{EmbeddingClient, EmbeddingInFlightSnapshot};
