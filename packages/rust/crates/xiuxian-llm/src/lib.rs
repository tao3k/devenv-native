//! Shared xiuxian LLM runtime primitives.

#[cfg(test)]
#[path = "../tests/unit/lib_policy.rs"]
mod rust_project_harness_gate;

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = rust_project_harness_gate::llm_rust_harness_config()
);

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
