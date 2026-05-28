//! Embeddable HTTP JSON router for BPMN workflow control.
//!
//! Start with `api`; request, response, and error DTOs stay leaf-owned.

mod activity_evidence;
mod api;
mod bpmn_source_admission;
mod control_projection;
mod control_trace;
#[path = "error/api.rs"]
mod error_api;
mod execution_graph;
mod llm_host_work_schedule;
#[path = "request/api.rs"]
mod request_api;
#[path = "response/api.rs"]
mod response_api;
mod routes;
mod state;
mod workflow_source_admission;

#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
pub use api::QianjiControlOpenAiCompatibleLlmWorkerCompleteHttpResponse;
#[cfg(any(
    all(feature = "duckdb", feature = "valkey", feature = "qianji-full"),
    test
))]
pub use api::QianjiControlOpenAiCompatibleLlmWorkerRunHttpResponse;
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
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiControlBpmnSourceAdmissionHttpRequest,
    QianjiControlBpmnSourceAdmissionHttpResponse, QianjiControlBpmnSourceHttpResponse,
    QianjiControlBpmnSourceMediaType, QianjiControlDiagnosticsHttpResponse,
    QianjiControlHistoryHttpResponse, QianjiControlRecoveryApplyHttpRequest,
    QianjiControlRecoveryApplyHttpResponse, QianjiControlRecoveryHttpResponse,
    QianjiControlRunSummaryHttpResponse, QianjiControlWorkflowSourceAdmissionHttpRequest,
    QianjiControlWorkflowSourceAdmissionHttpResponse, QianjiControlWorkflowSourceCompilerMode,
    qianji_bpmn_workflow_router,
};
