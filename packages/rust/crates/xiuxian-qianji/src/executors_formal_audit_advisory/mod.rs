//! Advisory-audit bridge from Qianji contract feedback into local prompt context plans.

#[path = "evidence.rs"]
mod evidence;
#[path = "facade.rs"]
mod facade;
#[path = "planning.rs"]
mod planning;
#[path = "prompt_context.rs"]
mod prompt_context;

#[cfg(feature = "advisory-prompt-pack-cache")]
pub use facade::QianjiAdvisoryPromptPackArtifactReport;
pub use facade::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
