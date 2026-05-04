//! Formal-audit LLM feature seam. Start in `api`.

#[path = "api.rs"]
mod api;
#[path = "context.rs"]
mod context;
#[path = "runtime.rs"]
mod runtime;
#[path = "scoring.rs"]
mod scoring;

pub use api::LlmAugmentedAuditMechanism;
