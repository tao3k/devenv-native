//! Activity schedule journal recording helpers.

use super::metadata::llm_activity_schedule_task;
use super::model::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityJournalWriteOutcome,
    ActivityStartedJournalRecord, AdmittedActivityScheduleRecord,
    AdmittedActivityTaskScheduleRecord, AdmittedLlmActivityScheduleRecord,
};
use super::transition::{
    find_existing_activity_event, validate_completion_transition, validate_failure,
    validate_failure_transition, validate_result, validate_schedule_transition,
    validate_start_transition, validate_started_record,
};
use crate::{ControlEventKind, ControlEventRecord, ControlLedger, ControlResult, replay_run_view};

/// Records an already admitted tool activity as an `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid or the ledger
/// append fails.
pub fn record_admitted_activity_schedule<L>(
    ledger: &L,
    request: AdmittedActivityScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    ledger.append_event(request.into_event())
}

/// Records an admitted activity schedule with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid, the activity
/// was already scheduled with different details, replay fails, or the ledger
/// append fails.
pub fn record_admitted_activity_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedActivityScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = request.admission.task.clone();
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_activity_schedule(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an already admitted workflow-neutral activity task as an
/// `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the task payload is invalid or the ledger
/// append fails.
pub fn record_admitted_activity_task_schedule<L>(
    ledger: &L,
    request: AdmittedActivityTaskScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.task.validate()?;
    ledger.append_event(request.into_event())
}

/// Records an admitted workflow-neutral activity task schedule with duplicate
/// and transition guards.
///
/// # Errors
///
/// Returns a control error when the task payload is invalid, the activity was
/// already scheduled with different details, replay fails, or the ledger append
/// fails.
pub fn record_admitted_activity_task_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedActivityTaskScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.task.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = request.task.clone();
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_activity_task_schedule(ledger, request)
        .map(ActivityJournalWriteOutcome::appended)
}

/// Records an already admitted LLM activity as an `ActivityScheduled` event.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid or the ledger
/// append fails.
pub fn record_admitted_llm_activity_schedule<L>(
    ledger: &L,
    request: AdmittedLlmActivityScheduleRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    ledger.append_event(request.into_event())
}

/// Records an admitted LLM activity schedule with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the admission payload is invalid, the activity
/// was already scheduled with different details, replay fails, or the ledger
/// append fails.
pub fn record_admitted_llm_activity_schedule_idempotent<L>(
    ledger: &L,
    request: AdmittedLlmActivityScheduleRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    request.admission.validate()?;
    let run_id = request.run_id.clone();
    let step_id = request.step_id.clone();
    let task = llm_activity_schedule_task(&request.admission);
    let kind = ControlEventKind::ActivityScheduled { task: task.clone() };
    let records = ledger.load_events(&run_id)?;
    if let Some(record) = find_existing_activity_event(&records, &run_id, step_id.as_ref(), &kind) {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_schedule_transition(&view, step_id.as_ref(), &task)?;
    record_admitted_llm_activity_schedule(ledger, request)
        .map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity attempt start as an `ActivityStarted` event.
///
/// # Errors
///
/// Returns a control error when the attempt is zero or the ledger append fails.
pub fn record_activity_started<L>(
    ledger: &L,
    request: ActivityStartedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_started_record(&request)?;
    ledger.append_event(request.into_event())
}

/// Records an activity attempt start with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the start record is invalid, the activity was
/// not scheduled, the transition is invalid, replay fails, or the ledger append
/// fails.
pub fn record_activity_started_idempotent<L>(
    ledger: &L,
    request: ActivityStartedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_started_record(&request)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let worker_id = request.worker_id.clone();
    let attempt = request.attempt;
    let kind = ControlEventKind::ActivityStarted {
        activity_id: activity_id.clone(),
        worker_id,
        attempt,
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_start_transition(&view, scope.step_id(), &activity_id, attempt)?;
    record_activity_started(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity completion as an `ActivityCompleted` event.
///
/// # Errors
///
/// Returns a control error when the result payload is invalid or the ledger
/// append fails.
pub fn record_activity_completed<L>(
    ledger: &L,
    request: ActivityCompletedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_result(&request.result)?;
    ledger.append_event(request.into_event())
}

/// Records an activity completion with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the result payload is invalid, the activity is
/// not in a started state, replay fails, or the ledger append fails.
pub fn record_activity_completed_idempotent<L>(
    ledger: &L,
    request: ActivityCompletedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_result(&request.result)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let kind = ControlEventKind::ActivityCompleted {
        activity_id: activity_id.clone(),
        result: request.result.clone(),
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_completion_transition(&view, scope.step_id(), &activity_id)?;
    record_activity_completed(ledger, request).map(ActivityJournalWriteOutcome::appended)
}

/// Records an activity failure as an `ActivityFailed` event.
///
/// # Errors
///
/// Returns a control error when the failure payload is invalid or the ledger
/// append fails.
pub fn record_activity_failed<L>(
    ledger: &L,
    request: ActivityFailedJournalRecord,
) -> ControlResult<ControlEventRecord>
where
    L: ControlLedger + ?Sized,
{
    validate_failure(&request.failure)?;
    ledger.append_event(request.into_event())
}

/// Records an activity failure with duplicate and transition guards.
///
/// # Errors
///
/// Returns a control error when the failure payload is invalid, the activity
/// state cannot accept the failure, replay fails, or the ledger append fails.
pub fn record_activity_failed_idempotent<L>(
    ledger: &L,
    request: ActivityFailedJournalRecord,
) -> ControlResult<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    validate_failure(&request.failure)?;
    let scope = request.scope.clone();
    let activity_id = request.activity_id.clone();
    let failure = request.failure.clone();
    let kind = ControlEventKind::ActivityFailed {
        activity_id: activity_id.clone(),
        failure: failure.clone(),
    };
    let records = ledger.load_events(scope.run_id())?;
    if let Some(record) =
        find_existing_activity_event(&records, scope.run_id(), scope.step_id(), &kind)
    {
        return Ok(ActivityJournalWriteOutcome::already_recorded(record));
    }
    let view = replay_run_view(records)?;
    validate_failure_transition(&view, scope.step_id(), &activity_id, &failure)?;
    record_activity_failed(ledger, request).map(ActivityJournalWriteOutcome::appended)
}
