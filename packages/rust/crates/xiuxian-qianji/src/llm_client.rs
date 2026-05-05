#[cfg(feature = "llm")]
/// Shared LLM client trait object type when `llm` feature is enabled.
pub type QianjiLlmClient = dyn xiuxian_llm::llm::LlmClient;

#[cfg(not(feature = "llm"))]
/// Placeholder trait object type when `llm` feature is disabled.
pub type QianjiLlmClient = dyn std::any::Any + Send + Sync;
