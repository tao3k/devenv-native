//! Cargo entry point for dormant `xiuxian-qianji` unit suites.

pub use xiuxian_qianji::runtime_config;
pub use xiuxian_qianji::{
    BpmnAdapterError, BpmnOrchestrationError, QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver,
    QianjiBpmnExecutionFacade, QianjiBpmnExecutionMode, QianjiBpmnExecutionRequest,
    QianjiBpmnExecutionScheduler, QianjiBpmnHostBridge, QianjiBpmnPendingHostWorkHttpResponse,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, QianjiBpmnWorkflowActionHttpRequest,
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowHttpErrorBody, QianjiBpmnWorkflowHttpState,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowInterruptRequest,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowSnapshotHttpResponse, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusHttpResponse,
    QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskClaimHttpResponse, QianjiBpmnWorkflowTaskClaimPayload,
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseRequest, QianjiBpmnWorkflowWorklistItem,
    QianjiBpmnWorkflowWorklistRequest, QianjiBpmnWorkflowWorklistRoutingFilter,
    SchedulerAgentIdentity, dispatch_pending_host_work_request, load_bpmn_package_from_files,
    qianji_bpmn_workflow_router, resolve_pending_host_work, resolve_waiting_external_event,
};

#[path = "unit/bpmn_engine_dependency.rs"]
mod bpmn_engine_dependency;
#[path = "unit/bpmn/mod.rs"]
mod bpmn_tests;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_adversarial_loop.rs"]
mod unit_adversarial_loop;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_execution.rs"]
mod unit_qianji_execution;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_safety.rs"]
mod unit_qianji_safety;
