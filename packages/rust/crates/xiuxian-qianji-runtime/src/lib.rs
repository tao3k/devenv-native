//! Durable runtime adapters for Qianji workflow execution.
//!
//! This crate is the dependency-safe execution boundary between BPMN workflow
//! semantics and the workflow-neutral Qianji control plane. It depends on
//! `xiuxian-qianji-bpmn-engine` for host-work facts and on `xiuxian-qianji-control`
//! for durable activity tasks. It does not depend on the CLI/server crate.

pub mod bpmn_host_work;
pub mod flowhub;
pub mod workflow_control;

pub use bpmn_host_work::{
    BPMN_HOST_WORK_ACTIVITY_METADATA_KEY, BPMN_HOST_WORK_ACTIVITY_SCHEMA,
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BPMN_HOST_WORK_COMPLETION_SCHEMA, BPMN_HOST_WORK_EVIDENCE_RUN_SCHEMA,
    BPMN_HOST_WORK_FAILURE_METADATA_KEY, BPMN_HOST_WORK_FAILURE_SCHEMA,
    BpmnHostWorkActivityEvidenceInput, BpmnHostWorkActivityScheduleInput, BpmnHostWorkCompletion,
    BpmnHostWorkCompletionActivityEvidenceInput, BpmnHostWorkCompletionKind, BpmnHostWorkFailure,
    BpmnHostWorkFailureActivityEvidenceInput, BpmnHostWorkIdentity,
    build_bpmn_host_work_activity_result, build_bpmn_host_work_activity_schedule_record,
    ensure_bpmn_host_work_activity_evidence_run, find_matching_bpmn_host_work,
    pending_bpmn_host_work_matches_identity, record_bpmn_host_work_completion_activity_evidence,
    record_bpmn_host_work_failure_activity_evidence,
};
pub use flowhub::{
    FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY, FLOWHUB_SERVICE_ACTIVITY_SCHEMA,
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FLOWHUB_SERVICE_COMPLETION_METADATA_KEY,
    FLOWHUB_SERVICE_COMPLETION_SCHEMA, FLOWHUB_SERVICE_WORKER_RUN_SCHEMA, FlowhubScenarioIdRef,
    FlowhubServiceActivityScheduleInput, FlowhubServiceTaskCompletion,
    FlowhubServiceWorkerLoopOutput, FlowhubServiceWorkerLoopRequest,
    FlowhubServiceWorkerLoopRuntime, FlowhubServiceWorkerStepOutput, QianjiRuntimeBpmnActivityId,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
    QianjiRuntimeInstantMs, QianjiRuntimeLeaseTtlMs, QianjiRuntimeWorkerIdRef,
    build_flowhub_service_activity_schedule_record, build_flowhub_service_task_completion,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, flowhub_service_task_bpmn_source_path,
    run_flowhub_service_worker_completion_loop,
};
pub use workflow_control::{
    QianjiRuntimeBpmnInstanceId, QianjiRuntimeBpmnSourcePath,
    QianjiRuntimeContinueUntilHumanBoundary, QianjiRuntimeDmnSourcePaths,
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowStatusView,
    QianjiRuntimeWorkflowTaskCompleteRequest, QianjiRuntimeWorkflowTaskCompletionKind,
    QianjiRuntimeWorkflowTaskCompletionPayload,
};
