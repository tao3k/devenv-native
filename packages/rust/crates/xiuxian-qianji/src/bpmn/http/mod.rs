//! Embeddable HTTP JSON router for BPMN workflow control.
//!
//! Start with `api`; request, response, and error DTOs stay leaf-owned.

mod api;
#[path = "error/api.rs"]
mod error_api;
#[path = "request/api.rs"]
mod request_api;
#[path = "response/api.rs"]
mod response_api;
mod routes;

pub use api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowSnapshotHttpResponse, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStatusHttpQuery, QianjiBpmnWorkflowStatusHttpResponse,
    qianji_bpmn_workflow_router,
};
