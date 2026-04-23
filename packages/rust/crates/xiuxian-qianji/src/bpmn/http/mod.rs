//! Embeddable HTTP JSON router for BPMN workflow control.
//!
//! `mod.rs` is interface-only for the HTTP transport slice.

mod api;
mod dto;
mod routes;

pub use api::{QianjiBpmnWorkflowHttpState, qianji_bpmn_workflow_router};
pub use dto::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowStatusHttpResponse,
};
