use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityJournalWriteStatus, ActivityStatus, AdmittedActivityTaskScheduleRecord,
    ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, StepId,
    record_admitted_activity_task_schedule_idempotent,
};

use crate::control::support::activity_task;

#[test]
fn idempotent_helper_records_workflow_neutral_activity_task_schedule() -> Result<(), Box<dyn Error>>
{
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-generic-activity-task-schedule")?;
    let step_id = StepId::new("stage-generic-task")?;
    let activity_id = ActivityId::new("activity-generic-task")?;
    let task = activity_task(activity_id.clone())?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record generic activity task schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Generic task stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    let request =
        AdmittedActivityTaskScheduleRecord::step(run_id.clone(), step_id.clone(), 3, task);
    let appended = record_admitted_activity_task_schedule_idempotent(&ledger, request.clone())?;
    let duplicate = record_admitted_activity_task_schedule_idempotent(&ledger, request)?;

    assert_eq!(appended.status, ActivityJournalWriteStatus::Appended);
    assert_eq!(
        duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let activity = step
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Scheduled);
    assert_eq!(ledger.load_events(&run_id)?.len(), 3);

    Ok(())
}

#[test]
fn idempotent_helper_rejects_conflicting_workflow_neutral_activity_task_schedule()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-conflicting-generic-task-schedule")?;
    let activity_id = ActivityId::new("activity-conflicting-generic-task")?;
    let original_task = activity_task(activity_id.clone())?;
    let conflicting_task = activity_task(activity_id)?.with_timeout_ms(45_000);

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "reject conflicting generic activity task schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_admitted_activity_task_schedule_idempotent(
        &ledger,
        AdmittedActivityTaskScheduleRecord::run(run_id.clone(), 2, original_task),
    )?;

    let Err(error) = record_admitted_activity_task_schedule_idempotent(
        &ledger,
        AdmittedActivityTaskScheduleRecord::run(run_id.clone(), 3, conflicting_task),
    ) else {
        return Err(io::Error::other("conflicting schedule should fail").into());
    };

    assert!(
        error.to_string().contains("already exists"),
        "unexpected error: {error}"
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 2);

    Ok(())
}
