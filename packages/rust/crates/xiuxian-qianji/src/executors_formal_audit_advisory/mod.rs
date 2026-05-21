//! Advisory-audit bridge from Qianji contract feedback into `Qianhuan`.

#[path = "evidence.rs"]
mod evidence;
#[path = "facade.rs"]
mod facade;
#[path = "planning.rs"]
mod planning;

pub use facade::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
