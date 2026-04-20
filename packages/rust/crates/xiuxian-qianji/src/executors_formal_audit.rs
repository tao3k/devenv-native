//! Skeptic node: performs formal audit on Analyzer output.

#[path = "executors/formal_audit/advisory.rs"]
mod advisory;
#[path = "executors/formal_audit/native.rs"]
mod native;

#[cfg(feature = "llm")]
#[path = "executors/formal_audit/live_advisory.rs"]
mod live_advisory;
#[cfg(feature = "llm")]
#[path = "executors/formal_audit/llm.rs"]
mod llm;

pub use advisory::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
pub use native::FormalAuditMechanism;

#[cfg(feature = "llm")]
pub use live_advisory::QianjiLlmAdvisoryAuditExecutor;
#[cfg(feature = "llm")]
pub use llm::LlmAugmentedAuditMechanism;
