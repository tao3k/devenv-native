//! Durable runtime adapters for Qianji workflow execution.
//!
//! This crate is the dependency-safe execution boundary between BPMN workflow
//! semantics and the workflow-neutral Qianji control plane. It depends on
//! `xiuxian-qianji-bpmn-engine` for host-work facts and on `xiuxian-qianji-control`
//! for durable activity tasks. It does not depend on the CLI/server crate.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public runtime adapter API"),
        )
    }
);

pub mod flowhub;
pub mod workflow_control;

pub use flowhub::{
    FLOWHUB_SERVICE_ACTIVITY_METADATA_KEY, FLOWHUB_SERVICE_ACTIVITY_SCHEMA,
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FLOWHUB_SERVICE_COMPLETION_METADATA_KEY,
    FLOWHUB_SERVICE_COMPLETION_SCHEMA, FlowhubScenarioIdRef, FlowhubServiceActivityScheduleInput,
    FlowhubServiceTaskCompletion, QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnInstanceIdRef,
    QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record, build_flowhub_service_task_completion,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, flowhub_service_task_bpmn_source_path,
};
pub use workflow_control::{
    QianjiRuntimeBpmnInstanceId, QianjiRuntimeBpmnSourcePath,
    QianjiRuntimeContinueUntilHumanBoundary, QianjiRuntimeDmnSourcePaths,
    QianjiRuntimeWorkflowControlPort, QianjiRuntimeWorkflowResumeRequest,
    QianjiRuntimeWorkflowStatusRequest, QianjiRuntimeWorkflowStatusView,
    QianjiRuntimeWorkflowTaskCompleteRequest, QianjiRuntimeWorkflowTaskCompletionKind,
    QianjiRuntimeWorkflowTaskCompletionPayload,
};
