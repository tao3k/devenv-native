//! BPMN host-work evidence adapter over the qianji-control ledger.

use std::path::Path;

use serde_json::{Value, json};
use xiuxian_qianji_bpmn_engine::PendingHostWork;
use xiuxian_qianji_control::{
    ActivityId, ActivityStatus, ControlError, ControlLedger, ControlResult, ErrorCode,
    RunCreatedJournalRecord, RunId, WorkerActivityCompletedRecord, WorkerActivityFailedRecord,
    WorkerActivityFailureInput, WorkerActivityStartRecord, WorkerActivityTask, WorkerId,
    record_admitted_activity_task_schedule_idempotent, record_run_created,
    record_worker_activity_completed_idempotent, record_worker_activity_failed_idempotent,
    record_worker_activity_started_idempotent,
};

use crate::{
    BpmnHostWorkActivityScheduleInput, BpmnHostWorkCompletion, QianjiRuntimeBpmnInstanceIdRef,
    QianjiRuntimeInstantMs, build_bpmn_host_work_activity_result,
    build_bpmn_host_work_activity_schedule_record,
};

/// Metadata schema for BPMN host-work evidence run creation.
pub const BPMN_HOST_WORK_EVIDENCE_RUN_SCHEMA: &str =
    "xiuxian_qianji.bpmn.host_work_evidence_run.v1";
/// Metadata key for BPMN host-work failure terminal events.
pub const BPMN_HOST_WORK_FAILURE_METADATA_KEY: &str = "qianji_bpmn_host_work_failure";
/// Metadata schema for BPMN host-work failure terminal events.
pub const BPMN_HOST_WORK_FAILURE_SCHEMA: &str = "xiuxian_qianji.bpmn.host_work_failure.v1";

/// Runtime input for one generic BPMN host-work activity-evidence boundary.
#[derive(Debug, Clone, Copy)]
pub struct BpmnHostWorkActivityEvidenceInput<'a> {
    /// Owning Qianji control-plane run id.
    pub run_id: &'a RunId,
    /// BPMN workflow instance id.
    pub instance_id: QianjiRuntimeBpmnInstanceIdRef<'a>,
    /// Source BPMN document path used by the workflow route.
    pub bpmn_source: &'a Path,
    /// Pending BPMN host work currently blocking the workflow.
    pub pending_work: &'a PendingHostWork,
    /// Worker identity used for the durable activity start event.
    pub worker_id: &'a WorkerId,
    /// Schedule timestamp supplied by the caller.
    pub scheduled_at_ms: QianjiRuntimeInstantMs,
    /// Worker start timestamp supplied by the caller.
    pub started_at_ms: QianjiRuntimeInstantMs,
    /// Worker terminal timestamp supplied by the caller.
    pub terminal_at_ms: QianjiRuntimeInstantMs,
}

/// Runtime input for recording successful BPMN host-work completion evidence.
#[derive(Debug, Clone, Copy)]
pub struct BpmnHostWorkCompletionActivityEvidenceInput<'a> {
    /// Shared activity-evidence boundary facts.
    pub evidence: BpmnHostWorkActivityEvidenceInput<'a>,
    /// Runtime-neutral completion facts.
    pub completion: &'a BpmnHostWorkCompletion,
}

/// Runtime input for recording failed BPMN host-work evidence.
#[derive(Debug, Clone)]
pub struct BpmnHostWorkFailureActivityEvidenceInput<'a> {
    /// Shared activity-evidence boundary facts.
    pub evidence: BpmnHostWorkActivityEvidenceInput<'a>,
    /// Runtime-neutral failure facts.
    pub failure: BpmnHostWorkFailure,
}

/// Runtime-neutral BPMN host-work failure facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnHostWorkFailure {
    /// Failure code recorded on the durable activity failure event.
    pub error_code: ErrorCode,
    /// Diagnostic failure message.
    pub message: String,
    /// Whether retry policy may schedule another attempt.
    pub retryable: bool,
    /// Extension metadata recorded on the durable failure event.
    pub metadata: Value,
}

/// Ensures a BPMN host-work evidence run exists.
///
/// # Errors
///
/// Returns a control error when replaying or appending the run-created event
/// fails.
pub fn ensure_bpmn_host_work_activity_evidence_run(
    ledger: &dyn ControlLedger,
    run_id: &RunId,
    occurred_at_ms: QianjiRuntimeInstantMs,
    instance_id: QianjiRuntimeBpmnInstanceIdRef<'_>,
) -> ControlResult<()> {
    if !ledger.load_events(run_id)?.is_empty() {
        return Ok(());
    }
    let request = RunCreatedJournalRecord::new(
        run_id.clone(),
        format!("BPMN host-work evidence for {}", instance_id.as_str()),
        occurred_at_ms.as_millis(),
    )
    .with_metadata(json!({
        "schema": BPMN_HOST_WORK_EVIDENCE_RUN_SCHEMA,
        "instanceId": instance_id.as_str()
    }));
    record_run_created(ledger, request).map(|_| ())
}

/// Records successful generic BPMN host-work completion activity evidence.
///
/// # Errors
///
/// Returns a control error when run creation, schedule recording, worker task
/// replay, start recording, completion result building, or terminal recording
/// fails.
pub fn record_bpmn_host_work_completion_activity_evidence(
    ledger: &dyn ControlLedger,
    input: BpmnHostWorkCompletionActivityEvidenceInput<'_>,
) -> ControlResult<()> {
    ensure_bpmn_host_work_activity_evidence_run(
        ledger,
        input.evidence.run_id,
        input.evidence.scheduled_at_ms,
        input.evidence.instance_id,
    )?;
    let Some(worker_task) = schedule_and_start_worker_task(ledger, input.evidence)? else {
        return Ok(());
    };
    let result = build_bpmn_host_work_activity_result(input.completion)?;
    record_worker_activity_completed_idempotent(
        ledger,
        WorkerActivityCompletedRecord::new(
            worker_task,
            input.evidence.terminal_at_ms.as_millis(),
            result,
        ),
    )?;
    Ok(())
}

/// Records failed generic BPMN host-work activity evidence.
///
/// # Errors
///
/// Returns a control error when failure validation, run creation, schedule
/// recording, worker task replay, start recording, or terminal recording
/// fails.
pub fn record_bpmn_host_work_failure_activity_evidence(
    ledger: &dyn ControlLedger,
    input: BpmnHostWorkFailureActivityEvidenceInput<'_>,
) -> ControlResult<()> {
    WorkerActivityFailureInput::validate_message(input.failure.message.trim())?;
    ensure_bpmn_host_work_activity_evidence_run(
        ledger,
        input.evidence.run_id,
        input.evidence.scheduled_at_ms,
        input.evidence.instance_id,
    )?;
    let Some(worker_task) = schedule_and_start_worker_task(ledger, input.evidence)? else {
        return Ok(());
    };
    let failure_input = WorkerActivityFailureInput::try_new(
        worker_task,
        input.failure.error_code,
        input.failure.message.trim().to_owned(),
    )?
    .with_failed_at_ms(input.evidence.terminal_at_ms.as_millis())
    .with_retryable(input.failure.retryable);
    let failure_record =
        WorkerActivityFailedRecord::try_new(failure_input)?.with_metadata(input.failure.metadata);
    record_worker_activity_failed_idempotent(ledger, failure_record)?;
    Ok(())
}

fn schedule_and_start_worker_task(
    ledger: &dyn ControlLedger,
    input: BpmnHostWorkActivityEvidenceInput<'_>,
) -> ControlResult<Option<WorkerActivityTask>> {
    let schedule_record =
        build_bpmn_host_work_activity_schedule_record(BpmnHostWorkActivityScheduleInput {
            run_id: input.run_id,
            occurred_at_ms: input.scheduled_at_ms,
            instance_id: input.instance_id,
            bpmn_source: input.bpmn_source,
            pending_work: input.pending_work,
        })?;
    let activity_id = schedule_record.task.activity_id.clone();
    record_admitted_activity_task_schedule_idempotent(ledger, schedule_record)?;
    let Some(worker_task) = load_scheduled_worker_task(ledger, input.run_id, &activity_id)? else {
        return Ok(None);
    };
    record_worker_activity_started_idempotent(
        ledger,
        WorkerActivityStartRecord::new(
            worker_task.clone(),
            input.worker_id.clone(),
            input.started_at_ms.as_millis(),
        ),
    )?;
    Ok(Some(worker_task))
}

fn load_scheduled_worker_task(
    ledger: &dyn ControlLedger,
    run_id: &RunId,
    activity_id: &ActivityId,
) -> ControlResult<Option<WorkerActivityTask>> {
    let worker_task = ledger
        .load_worker_activity_tasks(run_id, None)?
        .into_iter()
        .find(|task| &task.activity_id == activity_id);
    if worker_task.is_some() {
        return Ok(worker_task);
    }
    let view = ledger.load_run_view(run_id)?;
    if view.activities.get(activity_id).is_some_and(|activity| {
        matches!(
            activity.status,
            ActivityStatus::Completed | ActivityStatus::Failed
        )
    }) {
        return Ok(None);
    }
    Err(ControlError::InvalidEventSequence {
        message: "scheduled BPMN host-work activity task was not replayable".to_owned(),
    })
}
