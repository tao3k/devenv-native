//! Embedding transport primitives.

/// Backend mode parsing and normalized backend kinds.
#[path = "backend.rs"]
pub mod embedding_backend;
/// Memory embedding runtime guard (timeout/cooldown/dimension repair).
#[path = "runtime.rs"]
pub mod embedding_runtime;
/// OpenAI-compatible embedding transport utilities.
pub mod openai_compat;

pub use embedding_backend::{EmbeddingBackendKind, parse_embedding_backend_kind};
pub use embedding_runtime::{
    DEFAULT_MEMORY_EMBED_TIMEOUT, DEFAULT_MEMORY_EMBED_TIMEOUT_COOLDOWN,
    EMBEDDING_SOURCE_EMBEDDING, EMBEDDING_SOURCE_EMBEDDING_REPAIRED, EMBEDDING_SOURCE_UNAVAILABLE,
    EmbeddingRuntime, MAX_MEMORY_EMBED_COOLDOWN_MS, MAX_MEMORY_EMBED_TIMEOUT_MS,
    MIN_MEMORY_EMBED_TIMEOUT_MS, MemoryEmbeddingErrorKind, repair_embedding_dimension,
};
