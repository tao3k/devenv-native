//! Cargo entry point for dormant `xiuxian-qianji` unit suites.

pub use xiuxian_qianji::runtime_config;
pub use xiuxian_qianji::{
    BpmnAdapterError, BpmnOrchestrationError, FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef,
    FlowhubServiceActivityHttpScheduleInput, FlowhubServiceActivityScheduleInput,
    QianjiBpmnCheckpointStore, QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade,
    QianjiBpmnExecutionMode, QianjiBpmnExecutionRequest, QianjiBpmnExecutionScheduler,
    QianjiBpmnHostBridge, QianjiBpmnPendingHostWorkHttpResponse, QianjiBpmnSchedulerLeaseConfig,
    QianjiBpmnSession, QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowEventPollRequest,
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowHttpErrorBody,
    QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowInstancesRequest,
    QianjiBpmnWorkflowInterruptRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowRunHttpResponse, QianjiBpmnWorkflowSnapshotHttpResponse,
    QianjiBpmnWorkflowStartHttpRequest, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowStatusHttpResponse, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimHttpRequest, QianjiBpmnWorkflowTaskClaimHttpResponse,
    QianjiBpmnWorkflowTaskClaimPayload, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionHttpKind,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleaseHttpRequest, QianjiBpmnWorkflowTaskReleaseHttpResponse,
    QianjiBpmnWorkflowTaskReleasePayload, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistItem, QianjiBpmnWorkflowWorklistRequest,
    QianjiBpmnWorkflowWorklistRoutingFilter, QianjiRuntimeBpmnInstanceIdRef,
    QianjiRuntimeInstantMs, SchedulerAgentIdentity, build_flowhub_service_activity_schedule_record,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
    build_flowhub_service_task_completion_payload,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, dispatch_pending_host_work_request,
    load_bpmn_package_from_files, qianji_bpmn_workflow_router, resolve_pending_host_work,
    resolve_waiting_external_event,
};

#[path = "unit/bpmn_engine_dependency.rs"]
mod bpmn_engine_dependency;
#[path = "unit/bpmn/mod.rs"]
mod bpmn_tests;
#[path = "unit/lib_policy.rs"]
mod lib_policy;
#[path = "unit/support/valkey.rs"]
mod qianji_test_valkey_support;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_adversarial_loop.rs"]
mod unit_adversarial_loop;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_execution.rs"]
mod unit_qianji_execution;
#[cfg(feature = "qianji-full")]
#[path = "unit/unit_qianji_safety.rs"]
mod unit_qianji_safety;
