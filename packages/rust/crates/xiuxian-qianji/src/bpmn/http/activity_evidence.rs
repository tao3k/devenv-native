//! Durable `ActivityTask` evidence for qianji-server host-work completion.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{PendingHostWork, PendingHostWorkKind};
use xiuxian_qianji_control::{
    ControlLedger, ErrorCode, RunId, WorkerActivityFailureInput, WorkerId,
};
use xiuxian_qianji_runtime::{
    BPMN_HOST_WORK_FAILURE_METADATA_KEY, BPMN_HOST_WORK_FAILURE_SCHEMA,
    BpmnHostWorkActivityEvidenceInput, BpmnHostWorkCompletionActivityEvidenceInput,
    BpmnHostWorkFailure, BpmnHostWorkFailureActivityEvidenceInput, BpmnHostWorkIdentity,
    QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeBpmnProcessId,
    QianjiRuntimeBpmnTokenId, QianjiRuntimeInstantMs, find_matching_bpmn_host_work,
    record_bpmn_host_work_completion_activity_evidence,
    record_bpmn_host_work_failure_activity_evidence,
};

use super::error_api::QianjiBpmnWorkflowHttpError;
use super::request_api::{
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskFailureHttpPayload,
};
use crate::bpmn::control::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::host_work_activity_adapter::bpmn_host_work_completion_from_payload;
use crate::bpmn::identity::QianjiBpmnWorkflowInstanceId;

const QIANJI_SERVER_NATIVE_HOST_WORKER_ID: &str = "qianji-server.native-host";

#[derive(Clone, Copy)]
pub(super) struct QianjiBpmnWorkflowCompletionActivityEvidenceInput<'a> {
    pub(super) instance_id: &'a QianjiBpmnWorkflowInstanceId,
    pub(super) bpmn_path: &'a Path,
    pub(super) pending_work: &'a PendingHostWork,
    pub(super) completion: &'a QianjiBpmnWorkflowTaskCompletionPayload,
    pub(super) now_ms: u64,
}

#[derive(Clone, Copy)]
pub(super) struct QianjiBpmnWorkflowFailureActivityEvidenceInput<'a> {
    pub(super) instance_id: &'a QianjiBpmnWorkflowInstanceId,
    pub(super) bpmn_path: &'a Path,
    pub(super) pending_work: &'a PendingHostWork,
    pub(super) failure: &'a QianjiBpmnWorkflowTaskFailureHttpPayload,
    pub(super) now_ms: u64,
}

pub(super) fn record_completion_activity_evidence(
    ledger: &dyn ControlLedger,
    input: QianjiBpmnWorkflowCompletionActivityEvidenceInput<'_>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    let run_id = activity_evidence_run_id(input.instance_id)?;
    let worker_id = activity_evidence_worker_id()?;
    let completion = bpmn_host_work_completion_from_payload(input.completion);
    record_bpmn_host_work_completion_activity_evidence(
        ledger,
        BpmnHostWorkCompletionActivityEvidenceInput {
            evidence: activity_evidence_input(input, &run_id, &worker_id),
            completion: &completion,
        },
    )
    .map_err(activity_evidence_error)
}

pub(super) fn record_failure_activity_evidence(
    ledger: &dyn ControlLedger,
    input: QianjiBpmnWorkflowFailureActivityEvidenceInput<'_>,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    validate_failure_payload(input.failure)?;
    let run_id = activity_evidence_run_id(input.instance_id)?;
    let worker_id = activity_evidence_worker_id()?;
    let failure = bpmn_host_work_failure_from_payload(input.failure)?;
    record_bpmn_host_work_failure_activity_evidence(
        ledger,
        BpmnHostWorkFailureActivityEvidenceInput {
            evidence: failure_activity_evidence_input(input, &run_id, &worker_id),
            failure,
        },
    )
    .map_err(activity_evidence_error)
}

pub(super) fn matching_pending_work_for_completion(
    prepared: &QianjiBpmnPreparedWorkflowResume,
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> Option<PendingHostWork> {
    let identity = bpmn_host_work_identity_from_completion(completion);
    let checkpoint = prepared.loaded_checkpoint.as_ref()?;
    find_matching_bpmn_host_work(&checkpoint.state.pending_host_work, &identity).cloned()
}

pub(super) fn matching_pending_work_for_failure(
    prepared: &QianjiBpmnPreparedWorkflowResume,
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> Option<PendingHostWork> {
    let identity = bpmn_host_work_identity_from_failure(failure);
    let checkpoint = prepared.loaded_checkpoint.as_ref()?;
    find_matching_bpmn_host_work(&checkpoint.state.pending_host_work, &identity).cloned()
}

pub(super) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().try_into().unwrap_or(u64::MAX)
        })
}

fn activity_evidence_input<'a>(
    input: QianjiBpmnWorkflowCompletionActivityEvidenceInput<'a>,
    run_id: &'a RunId,
    worker_id: &'a WorkerId,
) -> BpmnHostWorkActivityEvidenceInput<'a> {
    BpmnHostWorkActivityEvidenceInput {
        run_id,
        instance_id: QianjiRuntimeBpmnInstanceIdRef::new(input.instance_id.as_str()),
        bpmn_source: input.bpmn_path,
        pending_work: input.pending_work,
        worker_id,
        scheduled_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms),
        started_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms + 1),
        terminal_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms + 2),
    }
}

fn failure_activity_evidence_input<'a>(
    input: QianjiBpmnWorkflowFailureActivityEvidenceInput<'a>,
    run_id: &'a RunId,
    worker_id: &'a WorkerId,
) -> BpmnHostWorkActivityEvidenceInput<'a> {
    BpmnHostWorkActivityEvidenceInput {
        run_id,
        instance_id: QianjiRuntimeBpmnInstanceIdRef::new(input.instance_id.as_str()),
        bpmn_source: input.bpmn_path,
        pending_work: input.pending_work,
        worker_id,
        scheduled_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms),
        started_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms + 1),
        terminal_at_ms: QianjiRuntimeInstantMs::from_millis(input.now_ms + 2),
    }
}

fn activity_evidence_run_id(
    instance_id: &QianjiBpmnWorkflowInstanceId,
) -> Result<RunId, QianjiBpmnWorkflowHttpError> {
    RunId::new(format!("bpmn.workflow.{}", instance_id.as_str()))
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))
}

fn activity_evidence_worker_id() -> Result<WorkerId, QianjiBpmnWorkflowHttpError> {
    WorkerId::new(QIANJI_SERVER_NATIVE_HOST_WORKER_ID)
        .map_err(|error| QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string()))
}

fn bpmn_host_work_failure_from_payload(
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> Result<BpmnHostWorkFailure, QianjiBpmnWorkflowHttpError> {
    let error_code = ErrorCode::new(failure.error_code.clone()).map_err(|_| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_task_failure",
            "failure.error_code must be a non-empty string",
        )
    })?;
    Ok(BpmnHostWorkFailure {
        error_code,
        message: failure.message.trim().to_owned(),
        retryable: failure.retryable,
        metadata: json!({
            BPMN_HOST_WORK_FAILURE_METADATA_KEY: {
                "schema": BPMN_HOST_WORK_FAILURE_SCHEMA,
                "tokenId": failure.token_id,
                "processId": failure.process_id.as_str(),
                "activityId": failure.activity_id.as_str(),
                "kind": failure_kind_name(failure.kind),
                "source": "qianji-server",
                "metadata": failure.metadata
            }
        }),
    })
}

fn validate_failure_payload(
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> Result<(), QianjiBpmnWorkflowHttpError> {
    ErrorCode::new(failure.error_code.clone()).map_err(|_| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_task_failure",
            "failure.error_code must be a non-empty string",
        )
    })?;
    WorkerActivityFailureInput::validate_message(failure.message.trim()).map_err(|_| {
        QianjiBpmnWorkflowHttpError::bad_request(
            "invalid_task_failure",
            "failure.message must be a non-empty string",
        )
    })
}

fn activity_evidence_error(error: impl std::fmt::Display) -> QianjiBpmnWorkflowHttpError {
    QianjiBpmnWorkflowHttpError::internal_server_error(error.to_string())
}

fn bpmn_host_work_identity_from_completion(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> BpmnHostWorkIdentity {
    BpmnHostWorkIdentity::new(
        QianjiRuntimeBpmnTokenId::new(completion.token_id),
        QianjiRuntimeBpmnProcessId::new(completion.process_id.as_str()),
        QianjiRuntimeBpmnActivityId::new(completion.activity_id.as_str()),
        pending_kind_from_completion_kind(completion.kind),
    )
}

fn bpmn_host_work_identity_from_failure(
    failure: &QianjiBpmnWorkflowTaskFailureHttpPayload,
) -> BpmnHostWorkIdentity {
    BpmnHostWorkIdentity::new(
        QianjiRuntimeBpmnTokenId::new(failure.token_id),
        QianjiRuntimeBpmnProcessId::new(failure.process_id.as_str()),
        QianjiRuntimeBpmnActivityId::new(failure.activity_id.as_str()),
        pending_kind_from_failure_kind(failure.kind),
    )
}

fn pending_kind_from_completion_kind(
    completion_kind: QianjiBpmnWorkflowTaskCompletionKind,
) -> PendingHostWorkKind {
    match completion_kind {
        QianjiBpmnWorkflowTaskCompletionKind::Task => PendingHostWorkKind::Task,
        QianjiBpmnWorkflowTaskCompletionKind::Send => PendingHostWorkKind::Send,
        QianjiBpmnWorkflowTaskCompletionKind::Service => PendingHostWorkKind::Service,
        QianjiBpmnWorkflowTaskCompletionKind::Script => PendingHostWorkKind::Script,
        QianjiBpmnWorkflowTaskCompletionKind::User => PendingHostWorkKind::User,
        QianjiBpmnWorkflowTaskCompletionKind::Manual => PendingHostWorkKind::Manual,
    }
}

fn pending_kind_from_failure_kind(
    failure_kind: QianjiBpmnWorkflowTaskCompletionHttpKind,
) -> PendingHostWorkKind {
    match failure_kind {
        QianjiBpmnWorkflowTaskCompletionHttpKind::Task => PendingHostWorkKind::Task,
        QianjiBpmnWorkflowTaskCompletionHttpKind::Send => PendingHostWorkKind::Send,
        QianjiBpmnWorkflowTaskCompletionHttpKind::Service => PendingHostWorkKind::Service,
        QianjiBpmnWorkflowTaskCompletionHttpKind::Script => PendingHostWorkKind::Script,
        QianjiBpmnWorkflowTaskCompletionHttpKind::User => PendingHostWorkKind::User,
        QianjiBpmnWorkflowTaskCompletionHttpKind::Manual => PendingHostWorkKind::Manual,
    }
}

fn failure_kind_name(kind: QianjiBpmnWorkflowTaskCompletionHttpKind) -> &'static str {
    match kind {
        QianjiBpmnWorkflowTaskCompletionHttpKind::Task => "task",
        QianjiBpmnWorkflowTaskCompletionHttpKind::Send => "send",
        QianjiBpmnWorkflowTaskCompletionHttpKind::Service => "service",
        QianjiBpmnWorkflowTaskCompletionHttpKind::Script => "script",
        QianjiBpmnWorkflowTaskCompletionHttpKind::User => "user",
        QianjiBpmnWorkflowTaskCompletionHttpKind::Manual => "manual",
    }
}
