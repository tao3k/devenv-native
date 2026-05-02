//! Shared xiuxian LLM runtime primitives.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

/// Embedding backend, OpenAI-compatible embedding, and memory embedding runtime.
pub mod embedding;
/// Chat LLM clients, provider adapters, multimodal helpers, and vision utilities.
pub mod llm;
/// Model-slot execution runtime and bus primitives.
#[path = "runtime/mod.rs"]
pub mod model_runtime;
#[doc(hidden)]
pub mod test_support;
/// Web crawling helpers used by LLM-facing retrieval flows.
pub mod web;
