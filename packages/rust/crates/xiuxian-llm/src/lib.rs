//! Shared xiuxian LLM runtime primitives.

/// Embedding backend, OpenAI-compatible embedding, and memory embedding runtime.
pub mod embedding;
/// Chat LLM clients, provider adapters, multimodal helpers, and vision utilities.
pub mod llm;
/// Model routing plane contracts and decision metadata.
#[cfg(feature = "model-routing")]
pub mod model_routing;
/// Model-slot execution runtime and bus primitives.
#[path = "runtime/mod.rs"]
pub mod model_runtime;
mod resource;
#[doc(hidden)]
pub mod test_support;
/// Web crawling helpers used by LLM-facing retrieval flows.
pub mod web;
