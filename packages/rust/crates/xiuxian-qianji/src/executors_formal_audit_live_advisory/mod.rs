//! Live LLM-backed advisory execution built on top of the advisory planning bridge.

#[path = "critique.rs"]
mod critique;
#[path = "runtime.rs"]
mod runtime;
use super::QianjiAdvisoryRolePlan;
#[path = "facade.rs"]
mod facade;

pub(super) use facade::DEFAULT_MODEL;
pub use facade::QianjiLlmAdvisoryAuditExecutor;
