use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityJournalScope, ActivityJournalWriteStatus, ActivityResult, ActivityStatus,
    ControlEvent, ControlEventKind, ControlLedger, ErrorCode, InMemoryControlLedger, RunId, StepId,
    WorkerActivityCompletedRecord, WorkerActivityFailedRecord, WorkerActivityFailureInput,
    WorkerActivityStartRecord, WorkerId, record_worker_activity_completed_idempotent,
    record_worker_activity_failed_idempotent, record_worker_activity_started_idempotent,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn worker_lifecycle_records_map_to_activity_journal_records() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-lifecycle-map")?;
    let step_id = StepId::new("step-worker-lifecycle-map")?;
    let activity_id = ActivityId::new("activity-worker-lifecycle-map")?;
    let worker_id = WorkerId::new("worker-lifecycle-map")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "worker lifecycle record mapping".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Worker mapping".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?,
        },
    ))?;
    let worker_task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing worker task"))?;

    let started = WorkerActivityStartRecord::new(worker_task.clone(), worker_id.clone(), 4)
        .into_activity_started_record();
    assert!(matches!(
        started.scope,
        ActivityJournalScope::Step { step_id: recorded, .. } if recorded == step_id
    ));
    assert_eq!(started.worker_id.as_ref(), Some(&worker_id));
    assert_eq!(started.activity_id, activity_id);
    assert_eq!(started.attempt, 1);

    let completed = WorkerActivityCompletedRecord::new(
        worker_task.clone(),
        5,
        ActivityResult {
            output_ref: Some(artifact_ref("artifact-worker-lifecycle-map-output")?),
            output_hash: Some("sha256:worker-lifecycle-map-output".to_owned()),
            metadata: serde_json::Value::Null,
        },
    )
    .into_activity_completed_record();
    assert_eq!(completed.activity_id, activity_id);
    assert_eq!(
        completed.result.output_hash.as_deref(),
        Some("sha256:worker-lifecycle-map-output")
    );

    let failed = WorkerActivityFailedRecord::new(
        WorkerActivityFailureInput::new(
            worker_task,
            ErrorCode::new("mapped_failure")?,
            "mapped worker failure",
        )
        .with_failed_at_ms(6)
        .with_retryable(true),
    )
    .with_metadata(serde_json::json!({"source": "mapping"}))
    .into_activity_failed_record();
    assert_eq!(failed.activity_id, activity_id);
    assert_eq!(failed.failure.attempt, 1);
    assert!(failed.failure.retryable);
    assert_eq!(
        failed
            .failure
            .metadata
            .get("source")
            .and_then(serde_json::Value::as_str),
        Some("mapping")
    );

    Ok(())
}

#[test]
fn worker_lifecycle_records_step_task_start_and_completion() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-lifecycle-step")?;
    let step_id = StepId::new("step-worker-lifecycle")?;
    let activity_id = ActivityId::new("activity-worker-lifecycle-step")?;
    let worker_id = WorkerId::new("worker-lifecycle-step")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "worker lifecycle step task".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Worker lifecycle".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?,
        },
    ))?;

    let worker_task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing worker task"))?;
    assert_eq!(worker_task.step_id.as_ref(), Some(&step_id));
    assert_eq!(worker_task.next_attempt, 1);

    let start = WorkerActivityStartRecord::new(worker_task.clone(), worker_id.clone(), 4);
    let start_appended = record_worker_activity_started_idempotent(&ledger, start.clone())?;
    let start_duplicate = record_worker_activity_started_idempotent(&ledger, start)?;
    assert_eq!(start_appended.status, ActivityJournalWriteStatus::Appended);
    assert_eq!(
        start_duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );

    let completion = WorkerActivityCompletedRecord::new(
        worker_task,
        5,
        ActivityResult {
            output_ref: Some(artifact_ref("artifact-worker-lifecycle-step-output")?),
            output_hash: Some("sha256:worker-lifecycle-step-output".to_owned()),
            metadata: serde_json::json!({"source": "worker_lifecycle"}),
        },
    );
    let complete_appended =
        record_worker_activity_completed_idempotent(&ledger, completion.clone())?;
    let complete_duplicate = record_worker_activity_completed_idempotent(&ledger, completion)?;
    assert_eq!(
        complete_appended.status,
        ActivityJournalWriteStatus::Appended
    );
    assert_eq!(
        complete_duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .steps
        .get(&step_id)
        .and_then(|step| step.activities.get(&activity_id))
        .ok_or_else(|| io::Error::other("missing replayed step activity"))?;
    assert_eq!(activity.status, ActivityStatus::Completed);
    assert_eq!(activity.worker_id.as_ref(), Some(&worker_id));
    assert_eq!(activity.attempt, 1);
    assert_eq!(
        activity
            .result
            .as_ref()
            .and_then(|result| result.output_hash.as_deref()),
        Some("sha256:worker-lifecycle-step-output")
    );

    Ok(())
}

#[test]
fn worker_lifecycle_failure_uses_worker_task_attempt() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-lifecycle-failure")?;
    let activity_id = ActivityId::new("activity-worker-lifecycle-failure")?;
    let worker_id = WorkerId::new("worker-lifecycle-failure")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "worker lifecycle failure".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?,
        },
    ))?;

    let worker_task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing worker task"))?;
    record_worker_activity_started_idempotent(
        &ledger,
        WorkerActivityStartRecord::new(worker_task.clone(), worker_id.clone(), 3),
    )?;

    let failure = WorkerActivityFailedRecord::new(
        WorkerActivityFailureInput::new(
            worker_task,
            ErrorCode::new("rate_limited")?,
            "provider rate limited worker request",
        )
        .with_failed_at_ms(4)
        .with_retryable(true),
    )
    .with_metadata(serde_json::json!({"provider": "llm.openai"}));
    let fail_appended = record_worker_activity_failed_idempotent(&ledger, failure.clone())?;
    let fail_duplicate = record_worker_activity_failed_idempotent(&ledger, failure)?;
    assert_eq!(fail_appended.status, ActivityJournalWriteStatus::Appended);
    assert_eq!(
        fail_duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed run activity"))?;
    assert_eq!(activity.status, ActivityStatus::Failed);
    assert_eq!(activity.worker_id.as_ref(), Some(&worker_id));
    assert_eq!(activity.attempt, 1);
    assert_eq!(
        activity.failure.as_ref().map(|failure| failure.retryable),
        Some(true)
    );
    assert_eq!(
        activity
            .failure
            .as_ref()
            .and_then(|failure| failure.metadata.get("provider"))
            .and_then(serde_json::Value::as_str),
        Some("llm.openai")
    );

    Ok(())
}

#[test]
fn worker_lifecycle_checked_failure_rejects_blank_message() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-lifecycle-blank-failure")?;
    let activity_id = ActivityId::new("activity-worker-lifecycle-blank-failure")?;
    let error_code = ErrorCode::new("blank_failure")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "worker lifecycle blank failure guard".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id)?,
        },
    ))?;
    let task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing worker task"))?;

    let Err(input_error) =
        WorkerActivityFailureInput::try_new(task.clone(), error_code.clone(), " ")
    else {
        return Err(io::Error::other("blank worker failure input should fail").into());
    };
    assert!(
        input_error
            .to_string()
            .contains("message must not be blank"),
        "unexpected input error: {input_error}"
    );
    let Err(message_error) = WorkerActivityFailureInput::validate_message("\t") else {
        return Err(io::Error::other("blank worker failure message should fail").into());
    };
    assert!(
        message_error
            .to_string()
            .contains("message must not be blank"),
        "unexpected message error: {message_error}"
    );

    let unchecked_input = WorkerActivityFailureInput::new(task, error_code, "");
    let Err(record_error) = WorkerActivityFailedRecord::try_new(unchecked_input) else {
        return Err(io::Error::other("blank worker failure record should fail").into());
    };
    assert!(
        record_error
            .to_string()
            .contains("message must not be blank"),
        "unexpected record error: {record_error}"
    );

    Ok(())
}

#[test]
fn worker_lifecycle_rejects_stale_failure_after_completion() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-worker-lifecycle-stale")?;
    let activity_id = ActivityId::new("activity-worker-lifecycle-stale")?;
    let worker_id = WorkerId::new("worker-lifecycle-stale")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "worker lifecycle stale task guard".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        2,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id)?,
        },
    ))?;

    let worker_task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("missing worker task"))?;
    record_worker_activity_started_idempotent(
        &ledger,
        WorkerActivityStartRecord::new(worker_task.clone(), worker_id, 3),
    )?;
    record_worker_activity_completed_idempotent(
        &ledger,
        WorkerActivityCompletedRecord::new(
            worker_task.clone(),
            4,
            ActivityResult {
                output_ref: None,
                output_hash: Some("sha256:worker-lifecycle-stale-output".to_owned()),
                metadata: serde_json::Value::Null,
            },
        ),
    )?;

    let Err(stale_failure) = record_worker_activity_failed_idempotent(
        &ledger,
        WorkerActivityFailedRecord::new(
            WorkerActivityFailureInput::new(
                worker_task,
                ErrorCode::new("late_failure")?,
                "stale worker task should not rewrite completion",
            )
            .with_failed_at_ms(5)
            .with_retryable(true),
        ),
    ) else {
        return Err(io::Error::other("stale failure should be rejected").into());
    };
    assert!(
        stale_failure.to_string().contains("completed"),
        "unexpected error: {stale_failure}"
    );

    Ok(())
}
