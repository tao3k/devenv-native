//! Scheduler public facade.
//!
//! Start in `types`; `core` executes the runtime while checkpoint, identity,
//! and policy expose the supporting public contracts.
mod core;
#[path = "../scheduler_execution.rs"]
mod execution;
#[path = "../scheduler_core_types.rs"]
mod types;

pub use self::types::{QianjiScheduler, SchedulerRuntimeServices};
pub use crate::{
    QianjiStateSnapshot, RoleAvailabilityRegistry, SchedulerAgentIdentity, SchedulerExecutionPolicy,
};
