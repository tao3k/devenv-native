//! Embeddable HTTP JSON router for BPMN workflow control.
//!
//! Start with `api`; request, response, and error DTOs stay leaf-owned.

mod activity_evidence;
mod api;
#[path = "error/api.rs"]
mod error_api;
#[path = "request/api.rs"]
mod request_api;
#[path = "response/api.rs"]
mod response_api;
mod routes;
mod state;

pub use api::{
    QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowHttpErrorBody, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowTaskClaimHttpPayload,
    QianjiBpmnWorkflowTaskClaimHttpRequest, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
    QianjiBpmnWorkflowTaskReleaseHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiControlDiagnosticsHttpResponse,
    QianjiControlHistoryHttpResponse, QianjiControlRecoveryApplyHttpRequest,
    QianjiControlRecoveryApplyHttpResponse, QianjiControlRecoveryHttpResponse,
    QianjiControlRunSummaryHttpResponse, qianji_bpmn_workflow_router,
};
