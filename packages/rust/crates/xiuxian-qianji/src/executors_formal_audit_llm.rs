//! Formal-audit LLM feature seam. Start in `api`.

#[path = "executors_formal_audit_llm/api.rs"]
mod api;
#[path = "executors_formal_audit_llm/context.rs"]
mod context;
#[path = "executors_formal_audit_llm/runtime.rs"]
mod runtime;
#[path = "executors_formal_audit_llm/scoring.rs"]
mod scoring;

pub use api::LlmAugmentedAuditMechanism;
