//! Embedding client runtime.
//!
//! Supports three backends:
//! - `http`: `/embed/batch` HTTP transport.
//! - `openai_http`: generic OpenAI-compatible `/v1/embeddings`.
//! - `litellm_rs`: Rust-native `LiteLLM` provider path (provider/API-key driven).

mod backend;
mod cache;
mod client;
mod test_api;
mod transport_http;
#[cfg(feature = "agent-provider-litellm")]
mod transport_litellm;
mod transport_openai;
mod types;

pub(crate) use backend::EmbeddingBackendMode;
pub(crate) use cache::EmbeddingCache;
pub use client::{EmbeddingClient, EmbeddingInFlightSnapshot};
#[cfg(feature = "agent-provider-litellm")]
pub(crate) use test_api::{
    TEST_OLLAMA_PLACEHOLDER_API_KEY, test_normalize_litellm_embedding_target,
    test_normalize_openai_compatible_base_url,
};
pub(crate) use test_api::{test_embed_http, test_parse_backend_mode};
