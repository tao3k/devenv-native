//! Public BPMN workflow HTTP router seam.
//!
//! This module re-exports the request, response, state, and error DTOs used by
//! embedders while keeping handler wiring inside the private routes module.

use super::routes;
use axum::Router;
use qianji_bpmn_engine::BpmnHostBridge;

pub use super::error_api::QianjiBpmnWorkflowHttpErrorBody;
pub use super::request_api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowTaskClaimHttpPayload, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteHttpRequest, QianjiBpmnWorkflowTaskCompletionHttpKind,
    QianjiBpmnWorkflowTaskCompletionHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpRequest,
};
pub use super::response_api::{
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskReleaseHttpResponse,
};
pub use super::state::QianjiBpmnWorkflowHttpState;

/// Builds an embeddable BPMN workflow-control HTTP router.
pub fn qianji_bpmn_workflow_router<H>(state: QianjiBpmnWorkflowHttpState<H>) -> Router
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    routes::router(state)
}
