mod idempotent;

use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityFailure, ActivityId,
    ActivityJournalScope, ActivityResult, ActivityStartedJournalRecord, ActivityStatus,
    ControlEvent, ControlEventKind, ControlLedger, ErrorCode, InMemoryControlLedger, RunId, StepId,
    WorkerId, record_activity_completed, record_activity_failed, record_activity_started,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn in_memory_ledger_replays_activity_journal_events() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-journal")?;
    let step_id = StepId::new("plan")?;
    let activity_id = ActivityId::new("activity-llm-plan")?;
    let worker_id = WorkerId::new("worker-llm-a")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "plan agent action".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Plan".to_owned(),
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
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        4,
        ControlEventKind::ActivityStarted {
            activity_id: activity_id.clone(),
            worker_id: Some(worker_id.clone()),
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        5,
        ControlEventKind::ActivityCompleted {
            activity_id: activity_id.clone(),
            result: ActivityResult {
                output_ref: Some(artifact_ref("artifact-llm-plan-output")?),
                output_hash: Some("sha256:plan-output".to_owned()),
                metadata: serde_json::Value::Null,
            },
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let activity = step
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Completed);
    assert_eq!(activity.worker_id.as_ref(), Some(&worker_id));
    assert_eq!(activity.attempt, 1);
    assert_eq!(
        activity.task.as_ref().map(|task| task.task_queue.as_str()),
        Some("llm.openai")
    );
    assert_eq!(
        activity
            .result
            .as_ref()
            .and_then(|result| result.output_hash.as_deref()),
        Some("sha256:plan-output")
    );

    Ok(())
}

#[test]
fn in_memory_ledger_replays_failed_activity_journal_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-failure")?;
    let activity_id = ActivityId::new("activity-tool-call")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "execute tool call".to_owned(),
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
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityFailed {
            activity_id: activity_id.clone(),
            failure: ActivityFailure {
                error_code: ErrorCode::new("rate_limited")?,
                message: "provider rejected request".to_owned(),
                retryable: true,
                attempt: 2,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Failed);
    assert_eq!(activity.attempt, 2);
    assert_eq!(
        activity.failure.as_ref().map(|failure| failure.retryable),
        Some(true)
    );

    Ok(())
}

#[test]
fn helper_records_activity_started_and_completed_events() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-lifecycle-helper")?;
    let step_id = StepId::new("stage-activity-lifecycle")?;
    let activity_id = ActivityId::new("activity-lifecycle-helper")?;
    let worker_id = WorkerId::new("worker-lifecycle")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record activity lifecycle through helpers".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Activity lifecycle".to_owned(),
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

    record_activity_started(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::step(run_id.clone(), step_id.clone()),
            4,
            activity_id.clone(),
            1,
        )
        .with_worker_id(worker_id.clone()),
    )?;
    record_activity_completed(
        &ledger,
        ActivityCompletedJournalRecord::new(
            ActivityJournalScope::step(run_id.clone(), step_id.clone()),
            5,
            activity_id.clone(),
            ActivityResult {
                output_ref: Some(artifact_ref("artifact-lifecycle-output")?),
                output_hash: Some("sha256:lifecycle-output".to_owned()),
                metadata: serde_json::Value::Null,
            },
        ),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let activity = step
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Completed);
    assert_eq!(activity.worker_id.as_ref(), Some(&worker_id));
    assert_eq!(activity.attempt, 1);
    assert_eq!(
        activity
            .result
            .as_ref()
            .and_then(|result| result.output_hash.as_deref()),
        Some("sha256:lifecycle-output")
    );

    Ok(())
}

#[test]
fn helper_records_activity_failed_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-failed-helper")?;
    let activity_id = ActivityId::new("activity-failed-helper")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record activity failure through helper".to_owned(),
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

    record_activity_failed(
        &ledger,
        ActivityFailedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            3,
            activity_id.clone(),
            ActivityFailure {
                error_code: ErrorCode::new("schema_invalid")?,
                message: "worker returned invalid schema".to_owned(),
                retryable: false,
                attempt: 1,
                metadata: serde_json::Value::Null,
            },
        ),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Failed);
    assert_eq!(activity.attempt, 1);
    assert_eq!(
        activity.failure.as_ref().map(|failure| failure.retryable),
        Some(false)
    );

    Ok(())
}

#[test]
fn helper_rejects_invalid_activity_lifecycle_records() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-invalid-activity-lifecycle")?;
    let activity_id = ActivityId::new("activity-invalid-lifecycle")?;

    let Err(start_error) = record_activity_started(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            1,
            activity_id.clone(),
            0,
        ),
    ) else {
        return Err(io::Error::other("zero start attempt should fail").into());
    };
    assert!(
        start_error.to_string().contains("attempt"),
        "unexpected error: {start_error}"
    );

    let Err(result_error) = record_activity_completed(
        &ledger,
        ActivityCompletedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            2,
            activity_id.clone(),
            ActivityResult {
                output_ref: None,
                output_hash: Some("   ".to_owned()),
                metadata: serde_json::Value::Null,
            },
        ),
    ) else {
        return Err(io::Error::other("blank result hash should fail").into());
    };
    assert!(
        result_error.to_string().contains("output_hash"),
        "unexpected error: {result_error}"
    );

    let Err(failure_error) = record_activity_failed(
        &ledger,
        ActivityFailedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            3,
            activity_id,
            ActivityFailure {
                error_code: ErrorCode::new("schema_invalid")?,
                message: " ".to_owned(),
                retryable: false,
                attempt: 0,
                metadata: serde_json::Value::Null,
            },
        ),
    ) else {
        return Err(io::Error::other("invalid failure should fail").into());
    };
    assert!(
        failure_error.to_string().contains("attempt"),
        "unexpected error: {failure_error}"
    );
    assert!(ledger.load_events(&run_id)?.is_empty());

    Ok(())
}
