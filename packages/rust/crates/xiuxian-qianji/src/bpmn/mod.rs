//! Host-owned BPMN facade for `xiuxian-qianji`.
//!
//! Start with `api`; it is the single visible entry seam for this feature.

#[path = "../bpmn_adapter_error.rs"]
mod adapter_error;
mod api;
#[path = "../bpmn_runtime_backend/mod.rs"]
mod backend;
#[path = "../bpmn_adapter_bridge.rs"]
mod bridge;
#[path = "control/api.rs"]
mod control;
#[path = "control/service/mod.rs"]
mod control_service;
#[path = "../bpmn_adapter_dispatch.rs"]
mod dispatch;
#[path = "../bpmn_runtime_driver/mod.rs"]
mod driver;
#[path = "../bpmn_runtime_error.rs"]
mod error;
#[path = "../bpmn_runtime_execution.rs"]
mod execution;
pub mod flowhub_activity_adapter;
pub mod host_work_activity_adapter;
#[path = "http/mod.rs"]
mod http_transport;
mod identity;
pub mod llm_activity_adapter;
#[path = "../bpmn_runtime_loader.rs"]
mod loader;
#[path = "../bpmn_runtime_ownership.rs"]
mod ownership;
#[cfg(feature = "duckdb")]
pub mod run_console_flight;
pub mod run_console_read_model;
#[path = "../bpmn_runtime_scheduler.rs"]
mod scheduler;
#[path = "../bpmn_runtime_session.rs"]
mod session;
#[path = "../bpmn_adapter_wait.rs"]
mod wait;

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
    BpmnAdapterError, BpmnOrchestrationError, BpmnUnsupportedStartNodeKind,
    DEFAULT_QIANJI_BPMN_SCHEDULER_LEASE_TTL_MS, QianjiBpmnActivityId, QianjiBpmnCheckpointStore,
    QianjiBpmnExecutionDriver, QianjiBpmnExecutionFacade, QianjiBpmnExecutionMode,
    QianjiBpmnExecutionReport, QianjiBpmnExecutionRequest, QianjiBpmnExecutionScheduler,
    QianjiBpmnHostBridge, QianjiBpmnHostBridgeBuilder, QianjiBpmnLeaseOwnerToken,
    QianjiBpmnPackageId, QianjiBpmnPendingHostCompletion, QianjiBpmnPendingHostWorkHttpResponse,
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnPreparedWorkflowStart, QianjiBpmnProcessId,
    QianjiBpmnSchedulerLeaseConfig, QianjiBpmnSession, QianjiBpmnStartAtNodeId,
    QianjiBpmnWorkflowActionHttpRequest, QianjiBpmnWorkflowCancelHttpResponse,
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiBpmnWorkflowEventPollReport,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowHttpCheckpointBackend,
    QianjiBpmnWorkflowHttpErrorBody, QianjiBpmnWorkflowHttpState, QianjiBpmnWorkflowInstanceId,
    QianjiBpmnWorkflowInstanceSummary, QianjiBpmnWorkflowInstancesReport,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowInterruptReport,
    QianjiBpmnWorkflowInterruptRequest, QianjiBpmnWorkflowResumeReport,
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowRunHttpResponse,
    QianjiBpmnWorkflowSnapshotHttpResponse, QianjiBpmnWorkflowStartHttpRequest,
    QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowStatusHttpQuery, QianjiBpmnWorkflowStatusHttpResponse,
    QianjiBpmnWorkflowStatusReport, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimHttpPayload, QianjiBpmnWorkflowTaskClaimHttpRequest,
    QianjiBpmnWorkflowTaskClaimHttpResponse, QianjiBpmnWorkflowTaskClaimPayload,
    QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteBatchHttpRequest, QianjiBpmnWorkflowTaskCompleteBatchReport,
    QianjiBpmnWorkflowTaskCompleteBatchRequest, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompleteReport, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleaseHttpPayload, QianjiBpmnWorkflowTaskReleaseHttpRequest,
    QianjiBpmnWorkflowTaskReleaseHttpResponse, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseReport, QianjiBpmnWorkflowTaskReleaseRequest,
    QianjiBpmnWorkflowWorklistItem, QianjiBpmnWorkflowWorklistReport,
    QianjiBpmnWorkflowWorklistRequest, QianjiBpmnWorkflowWorklistRoutingFilter,
    QianjiControlBpmnSourceAdmissionHttpRequest, QianjiControlBpmnSourceAdmissionHttpResponse,
    QianjiControlBpmnSourceHttpResponse, QianjiControlBpmnSourceMediaType,
    QianjiControlDiagnosticsHttpResponse, QianjiControlHistoryHttpResponse,
    QianjiControlRecoveryApplyHttpRequest, QianjiControlRecoveryApplyHttpResponse,
    QianjiControlRecoveryHttpResponse, QianjiControlRunSummaryHttpResponse,
    dispatch_pending_host_work_request, dispatch_pending_host_work_requests,
    load_bpmn_package_from_files, load_bpmn_package_from_files_with_options,
    qianji_bpmn_workflow_router, resolve_pending_host_work, resolve_waiting_external_event,
};
#[cfg(feature = "duckdb")]
pub use api::{
    DEFAULT_QIANJI_BPMN_DUCKDB_THREADS, QIANJI_BPMN_WORKFLOW_STATE_RECORD_KEY,
    QianjiBpmnDataRecord, QianjiBpmnDataStoreError, QianjiBpmnDuckDbDataStore,
    QianjiBpmnDuckDbDataStoreConfig,
};
pub use flowhub_activity_adapter::{
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FLOWHUB_SERVICE_COMPLETION_METADATA_KEY, FlowhubScenarioIdRef,
    FlowhubServiceActivityHttpScheduleInput, FlowhubServiceActivityScheduleInput,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
    build_flowhub_service_task_completion_payload,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data,
};
pub use host_work_activity_adapter::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BPMN_HOST_WORK_COMPLETION_SCHEMA, BpmnHostWorkActivityScheduleInput,
    build_bpmn_host_work_activity_result, build_bpmn_host_work_activity_schedule_record,
};
pub use llm_activity_adapter::{
    BPMN_HOST_WORK_LLM_ACTIVITY_ROUTE_SCHEMA, BpmnHostWorkLlmActivityRouteInput,
    BpmnHostWorkLlmEndpointDecision, BpmnHostWorkLlmRouteDecision,
    build_bpmn_host_work_llm_activity_route,
};
#[cfg(feature = "duckdb")]
pub use run_console_flight::{QIANJI_RUN_CONSOLE_RUN_ID_HEADER, QianjiRunConsoleFlightService};
pub use run_console_read_model::{
    QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE, QIANJI_RUN_CONSOLE_EVENT_ROUTE,
    QIANJI_RUN_CONSOLE_SCHEMA_VERSION, QianjiRunConsoleElementState,
};
#[cfg(feature = "duckdb")]
pub use run_console_read_model::{
    QianjiRunConsoleArrowReadModel, qianji_run_console_arrow_read_model,
    qianji_run_console_element_state_arrow_contract, qianji_run_console_element_state_arrow_schema,
    qianji_run_console_event_arrow_contract, qianji_run_console_event_arrow_schema,
};

#[cfg(test)]
#[path = "../../tests/unit/bpmn/mod.rs"]
mod tests;
