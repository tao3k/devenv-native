//! Host-owned BPMN execution driver facade.
//!
//! Start with `api`; the execution leaf owns runtime execution methods.

#[path = "bpmn_runtime_driver/api.rs"]
mod api;
#[path = "bpmn_runtime_driver/execution/api.rs"]
mod execution;

pub(in crate::bpmn) use api::{QianjiBpmnCheckpointLifecycle, QianjiBpmnHostCompletionAdvance};
pub use api::{
    QianjiBpmnExecutionDriver, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnPendingHostCompletion,
};
