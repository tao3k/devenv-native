//! Llm client surface for `xiuxian-qianji`.

/// Compatibility namespace boundary: this feature-gated alias preserves the
/// same public name as the no-`llm` placeholder.
///
/// Shared LLM client trait object type when `llm` feature is enabled.
#[cfg(feature = "llm")]
pub type QianjiLlmClient = dyn xiuxian_llm::llm::LlmClient;

/// Compatibility namespace boundary: this placeholder preserves the same
/// public name as the feature-enabled LLM client alias.
///
/// Placeholder trait object type when `llm` feature is disabled.
#[cfg(not(feature = "llm"))]
pub type QianjiLlmClient = dyn std::any::Any + Send + Sync;
