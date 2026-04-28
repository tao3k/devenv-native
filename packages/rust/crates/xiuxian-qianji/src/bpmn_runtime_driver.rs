//! Host-owned BPMN execution driver facade.
//!
//! Start with `api`; the driver leaf owns runtime execution methods.

#[path = "bpmn_runtime_driver/api.rs"]
mod api;
#[path = "bpmn_runtime_driver/driver/api.rs"]
mod driver;

pub(in crate::bpmn) use api::{QianjiBpmnCheckpointLifecycle, QianjiBpmnHostCompletionAdvance};
pub use api::{
    QianjiBpmnExecutionDriver, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnPendingHostCompletion,
};
