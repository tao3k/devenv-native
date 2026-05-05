//! Advisory-audit bridge from Qianji contract feedback into `Qianhuan`.

#[path = "facade.rs"]
mod facade;
#[path = "helpers.rs"]
mod helpers;
#[path = "planning.rs"]
mod planning;

pub use facade::{
    QianjiAdvisoryAuditExecutor, QianjiAdvisoryExecutionPlan, QianjiAdvisoryRolePlan,
};
