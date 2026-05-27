//! Activity journal idempotency and transition helpers.

use super::model::ActivityStartedJournalRecord;
use crate::{
    ActivityFailure, ActivityId, ActivityResult, ActivityStatus, ActivityTask, ActivityView,
    ControlError, ControlEventKind, ControlEventRecord, ControlResult, RunId, RunView, StepId,
};

pub(super) fn find_existing_activity_event(
    records: &[ControlEventRecord],
    run_id: &RunId,
    step_id: Option<&StepId>,
    kind: &ControlEventKind,
) -> Option<ControlEventRecord> {
    records
        .iter()
        .find(|record| {
            &record.event.run_id == run_id
                && record.event.step_id.as_ref() == step_id
                && &record.event.kind == kind
        })
        .cloned()
}

pub(super) fn validate_schedule_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    task: &ActivityTask,
) -> ControlResult<()> {
    if activity_for_scope(view, step_id, &task.activity_id).is_some() {
        return Err(invalid_activity_journal(
            "activity schedule already exists for activity_id with different lifecycle history",
        ));
    }
    Ok(())
}

pub(super) fn validate_start_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    attempt: u32,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "start")?;
    match activity.status {
        ActivityStatus::Scheduled => Ok(()),
        ActivityStatus::Failed if attempt > activity.attempt => Ok(()),
        ActivityStatus::Started => Err(invalid_activity_journal(
            "activity start is already in progress; duplicate starts must match an existing event",
        )),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity start cannot follow a completed activity",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity retry start attempt must be greater than the failed attempt",
        )),
        ActivityStatus::Pending => Err(invalid_activity_journal(
            "activity start requires a scheduled activity",
        )),
    }
}

pub(super) fn validate_completion_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "completion")?;
    match activity.status {
        ActivityStatus::Started => Ok(()),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity completion is already recorded with different result",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity completion cannot follow a failed activity",
        )),
        ActivityStatus::Pending | ActivityStatus::Scheduled => Err(invalid_activity_journal(
            "activity completion requires a started activity",
        )),
    }
}

pub(super) fn validate_failure_transition(
    view: &RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    failure: &ActivityFailure,
) -> ControlResult<()> {
    let activity = required_activity_for_scope(view, step_id, activity_id, "failure")?;
    match activity.status {
        ActivityStatus::Scheduled => Ok(()),
        ActivityStatus::Started if failure.attempt == activity.attempt => Ok(()),
        ActivityStatus::Started => Err(invalid_activity_journal(
            "activity failure attempt must match the started attempt",
        )),
        ActivityStatus::Failed => Err(invalid_activity_journal(
            "activity failure is already recorded with different payload",
        )),
        ActivityStatus::Completed => Err(invalid_activity_journal(
            "activity failure cannot follow a completed activity",
        )),
        ActivityStatus::Pending => Err(invalid_activity_journal(
            "activity failure requires a scheduled activity",
        )),
    }
}

fn required_activity_for_scope<'a>(
    view: &'a RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
    transition: &str,
) -> ControlResult<&'a ActivityView> {
    activity_for_scope(view, step_id, activity_id).ok_or_else(|| {
        invalid_activity_journal(&format!(
            "activity {transition} requires a scheduled activity"
        ))
    })
}

fn activity_for_scope<'a>(
    view: &'a RunView,
    step_id: Option<&StepId>,
    activity_id: &ActivityId,
) -> Option<&'a ActivityView> {
    match step_id {
        Some(step_id) => view
            .steps
            .get(step_id)
            .and_then(|step| step.activities.get(activity_id)),
        None => view.activities.get(activity_id),
    }
}

pub(super) fn validate_started_record(request: &ActivityStartedJournalRecord) -> ControlResult<()> {
    if request.attempt == 0 {
        return Err(invalid_activity_journal(
            "activity started attempt must be at least 1",
        ));
    }
    Ok(())
}

pub(super) fn validate_result(result: &ActivityResult) -> ControlResult<()> {
    if result
        .output_hash
        .as_ref()
        .is_some_and(|hash| hash.trim().is_empty())
    {
        return Err(invalid_activity_journal(
            "activity result output_hash must not be blank when supplied",
        ));
    }
    Ok(())
}

pub(super) fn validate_failure(failure: &ActivityFailure) -> ControlResult<()> {
    if failure.attempt == 0 {
        return Err(invalid_activity_journal(
            "activity failure attempt must be at least 1",
        ));
    }
    if failure.message.trim().is_empty() {
        return Err(invalid_activity_journal(
            "activity failure message must not be blank",
        ));
    }
    Ok(())
}

fn invalid_activity_journal(message: &str) -> ControlError {
    ControlError::InvalidEventSequence {
        message: message.to_owned(),
    }
}
