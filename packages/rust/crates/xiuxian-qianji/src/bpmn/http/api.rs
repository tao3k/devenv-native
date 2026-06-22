//! Public BPMN workflow HTTP router seam.
//!
//! This module re-exports the request, response, state, and error DTOs used by
//! embedders while keeping handler wiring inside the private routes module.

use super::routes;
use axum::Router;
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;

pub use super::error_api::QianjiBpmnWorkflowHttpErrorBody;
pub use super::request_api::{
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowTaskClaimHttpPayload, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiControlBpmnSourceAdmissionHttpRequest, QianjiControlRecoveryApplyHttpRequest,
    QianjiControlWorkflowSourceAdmissionHttpRequest, QianjiControlWorkflowSourceCompilerMode,
};
pub use super::response_api::{
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiControlBpmnSourceAdmissionHttpResponse,
    QianjiControlBpmnSourceHttpResponse, QianjiControlBpmnSourceMediaType,
    QianjiControlDiagnosticsHttpResponse, QianjiControlHistoryHttpResponse,
    QianjiControlRecoveryApplyHttpResponse, QianjiControlRecoveryHttpResponse,
    QianjiControlRunSummaryHttpResponse, QianjiControlWorkflowSourceAdmissionHttpResponse,
    QianjiControlWorkflowSourceAdmittedHttpResponse,
    QianjiControlWorkflowSourceRepairStartedHttpResponse,
};
pub use super::source_authoring::QianjiControlWorkflowSourceAuthoringMediaType;
pub use super::state::QianjiBpmnWorkflowHttpState;

/// Builds an embeddable BPMN workflow-control HTTP router.
pub fn qianji_bpmn_workflow_router<H>(state: QianjiBpmnWorkflowHttpState<H>) -> Router
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    routes::router(state)
}
