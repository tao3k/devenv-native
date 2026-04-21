//! Skeptic node: performs formal audit on Analyzer output.

#[path = "executors_formal_audit_advisory.rs"]
mod advisory;
#[path = "executors_formal_audit_native.rs"]
mod native;

#[cfg(feature = "llm")]
#[path = "executors_formal_audit_live_advisory.rs"]
mod live_advisory;
#[cfg(feature = "llm")]
#[path = "executors_formal_audit_llm.rs"]
mod llm;

pub use advisory::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
pub use native::FormalAuditMechanism;

#[cfg(feature = "llm")]
pub use live_advisory::QianjiLlmAdvisoryAuditExecutor;
#[cfg(feature = "llm")]
pub use llm::LlmAugmentedAuditMechanism;
