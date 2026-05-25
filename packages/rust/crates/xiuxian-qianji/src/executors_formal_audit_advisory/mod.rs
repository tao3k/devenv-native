//! Advisory-audit bridge from Qianji contract feedback into `Qianhuan`.

#[path = "evidence.rs"]
mod evidence;
#[path = "facade.rs"]
mod facade;
#[path = "planning.rs"]
mod planning;

#[cfg(feature = "advisory-prompt-pack-cache")]
pub use facade::QianjiAdvisoryPromptPackArtifactReport;
pub use facade::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
