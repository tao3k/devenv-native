use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ControlEvent, ControlEventKind, ControlLedger,
    IdempotencyKey, InMemoryControlLedger, RecoveryItemScope, RunId, StepId, TaskQueue,
};

#[test]
fn activity_queue_projection_selects_only_scheduled_tasks() -> Result<(), Box<dyn Error>> {
    let ledger = activity_queue_fixture()?;
    let run_id = RunId::new("run-activity-queue")?;
    let projection = ledger.load_activity_queue_projection(&run_id, None)?;

    assert_eq!(projection.run_id, run_id);
    assert_eq!(projection.task_queue, None);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(
        projection.items[0].activity.activity_id,
        ActivityId::new("activity-run-scheduled")?
    );
    assert_eq!(projection.items[0].scope, RecoveryItemScope::run());
    assert_eq!(
        projection.items[1].activity.activity_id,
        ActivityId::new("activity-step-scheduled")?
    );
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(StepId::new("step-activity-queue")?)
    );
    Ok(())
}

#[test]
fn activity_queue_projection_filters_by_task_queue() -> Result<(), Box<dyn Error>> {
    let ledger = activity_queue_fixture()?;
    let run_id = RunId::new("run-activity-queue")?;
    let task_queue = TaskQueue::new("tool.github")?;
    let projection = ledger.load_activity_queue_projection(&run_id, Some(&task_queue))?;

    assert_eq!(projection.task_queue, Some(task_queue));
    assert_eq!(projection.items.len(), 1);
    assert_eq!(
        projection.items[0].activity.activity_id,
        ActivityId::new("activity-step-scheduled")?
    );
    Ok(())
}

fn activity_queue_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-queue")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "activity queue projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        StepId::new("step-activity-queue")?,
        2,
        ControlEventKind::StepCreated {
            title: "Dispatch tool work".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-run-scheduled", "llm.plan", "llm.openai")?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        4,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-run-started", "llm.plan", "llm.openai")?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        5,
        ControlEventKind::ActivityStarted {
            activity_id: ActivityId::new("activity-run-started")?,
            worker_id: None,
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id,
        StepId::new("step-activity-queue")?,
        6,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-step-scheduled", "tool.github", "tool.github")?,
        },
    ))?;
    Ok(ledger)
}

fn activity_task(
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        ActivityId::new(activity_id)?,
        ActivityType::new(activity_type)?,
        TaskQueue::new(task_queue)?,
        IdempotencyKey::new(format!("{activity_id}/key"))?,
    ))
}
