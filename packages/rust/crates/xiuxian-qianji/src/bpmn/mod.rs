//! Host-owned BPMN facade for `xiuxian-qianji`.
//!
//! Start with `api`; it is the single visible entry seam for this feature.

#[path = "../bpmn_adapter_error.rs"]
mod adapter_error;
mod api;
#[path = "../bpmn_runtime_backend.rs"]
mod backend;
#[path = "../bpmn_adapter_bridge.rs"]
mod bridge;
#[path = "control/api.rs"]
mod control;
#[path = "control/service/mod.rs"]
mod control_service;
#[path = "../bpmn_adapter_dispatch.rs"]
mod dispatch;
#[path = "../bpmn_runtime_driver.rs"]
mod driver;
#[path = "../bpmn_runtime_error.rs"]
mod error;
#[path = "../bpmn_runtime_execution.rs"]
mod execution;
#[path = "http/mod.rs"]
mod http_transport;
#[path = "../bpmn_runtime_loader.rs"]
mod loader;
#[path = "../bpmn_runtime_ownership.rs"]
mod ownership;
#[path = "../bpmn_runtime_scheduler.rs"]
mod scheduler;
#[path = "../bpmn_runtime_session.rs"]
mod session;
#[path = "../bpmn_adapter_wait.rs"]
mod wait;

pub use api::{
    BpmnAdapterError, BpmnOrchestrationError, DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS,
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade,
    QianjiBpmnExecutionMode, QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder,
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowCancelHttpResponse, QianjiBpmnWorkflowCancelReport,
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollReport, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowResumeReport, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStartReport,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusHttpQuery,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskCompleteReport,
    QianjiBpmnWorkflowTaskCompleteRequest, dispatch_pending_host_work_request,
    dispatch_pending_host_work_requests, load_bpmn_package_from_files,
    load_bpmn_package_from_files_with_options, qianji_bpmn_workflow_router,
    resolve_pending_host_work, resolve_waiting_external_event,
};

#[cfg(test)]
#[path = "../../tests/unit/bpmn/mod.rs"]
mod tests;
