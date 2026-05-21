use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityFailure, ActivityId,
    ActivityJournalScope, ActivityJournalWriteStatus, ActivityResult, ActivityStartedJournalRecord,
    ActivityStatus, ControlEvent, ControlEventKind, ControlLedger, ErrorCode,
    InMemoryControlLedger, RunId, StepId, WorkerId, record_activity_completed_idempotent,
    record_activity_failed_idempotent, record_activity_started_idempotent,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn idempotent_lifecycle_helpers_return_existing_records() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-idempotent-activity-lifecycle")?;
    let step_id = StepId::new("stage-idempotent-activity")?;
    let activity_id = ActivityId::new("activity-idempotent-lifecycle")?;
    let worker_id = WorkerId::new("worker-idempotent")?;
    let scope = ActivityJournalScope::step(run_id.clone(), step_id.clone());

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record idempotent activity lifecycle".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Idempotent lifecycle".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id,
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?,
        },
    ))?;

    let start_request = ActivityStartedJournalRecord::new(scope.clone(), 4, activity_id.clone(), 1)
        .with_worker_id(worker_id);
    let start_appended = record_activity_started_idempotent(&ledger, start_request.clone())?;
    let start_duplicate = record_activity_started_idempotent(&ledger, start_request)?;
    assert_eq!(start_appended.status, ActivityJournalWriteStatus::Appended);
    assert_eq!(
        start_duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );
    assert_eq!(
        start_duplicate.record.sequence,
        start_appended.record.sequence
    );

    let result = ActivityResult {
        output_ref: Some(artifact_ref("artifact-idempotent-output")?),
        output_hash: Some("sha256:idempotent-output".to_owned()),
        metadata: serde_json::Value::Null,
    };
    let complete_request =
        ActivityCompletedJournalRecord::new(scope, 5, activity_id.clone(), result);
    let complete_appended =
        record_activity_completed_idempotent(&ledger, complete_request.clone())?;
    let complete_duplicate = record_activity_completed_idempotent(&ledger, complete_request)?;
    assert_eq!(
        complete_appended.status,
        ActivityJournalWriteStatus::Appended
    );
    assert_eq!(
        complete_duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );
    assert_eq!(
        complete_duplicate.record.sequence,
        complete_appended.record.sequence
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 5);

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .steps
        .values()
        .next()
        .and_then(|step| step.activities.get(&activity_id))
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;
    assert_eq!(activity.status, ActivityStatus::Completed);

    Ok(())
}

#[test]
fn idempotent_lifecycle_helpers_reject_invalid_transitions() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-invalid-idempotent-lifecycle")?;
    let activity_id = ActivityId::new("activity-invalid-idempotent")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "reject invalid idempotent lifecycle".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;

    let Err(start_before_schedule) = record_activity_started_idempotent(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            2,
            activity_id.clone(),
            1,
        ),
    ) else {
        return Err(io::Error::other("start before schedule should fail").into());
    };
    assert!(
        start_before_schedule.to_string().contains("scheduled"),
        "unexpected error: {start_before_schedule}"
    );

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task(activity_id.clone())?,
        },
    ))?;
    let Err(complete_before_start) = record_activity_completed_idempotent(
        &ledger,
        ActivityCompletedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            4,
            activity_id.clone(),
            ActivityResult {
                output_ref: None,
                output_hash: Some("sha256:too-early".to_owned()),
                metadata: serde_json::Value::Null,
            },
        ),
    ) else {
        return Err(io::Error::other("complete before start should fail").into());
    };
    assert!(
        complete_before_start.to_string().contains("started"),
        "unexpected error: {complete_before_start}"
    );

    record_activity_started_idempotent(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            5,
            activity_id.clone(),
            1,
        ),
    )?;
    record_activity_completed_idempotent(
        &ledger,
        ActivityCompletedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            6,
            activity_id.clone(),
            ActivityResult {
                output_ref: None,
                output_hash: Some("sha256:completed".to_owned()),
                metadata: serde_json::Value::Null,
            },
        ),
    )?;
    let Err(fail_after_complete) = record_activity_failed_idempotent(
        &ledger,
        ActivityFailedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            7,
            activity_id,
            ActivityFailure {
                error_code: ErrorCode::new("late_failure")?,
                message: "late failure should not rewrite completion".to_owned(),
                retryable: true,
                attempt: 1,
                metadata: serde_json::Value::Null,
            },
        ),
    ) else {
        return Err(io::Error::other("failure after completion should fail").into());
    };
    assert!(
        fail_after_complete.to_string().contains("completed"),
        "unexpected error: {fail_after_complete}"
    );

    Ok(())
}

#[test]
fn idempotent_lifecycle_helper_allows_retry_start_after_failure() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-idempotent-retry-start")?;
    let activity_id = ActivityId::new("activity-retry-start")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "allow retry start after failure".to_owned(),
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
    record_activity_started_idempotent(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            3,
            activity_id.clone(),
            1,
        ),
    )?;
    record_activity_failed_idempotent(
        &ledger,
        ActivityFailedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            4,
            activity_id.clone(),
            ActivityFailure {
                error_code: ErrorCode::new("rate_limited")?,
                message: "provider rate limited the first attempt".to_owned(),
                retryable: true,
                attempt: 1,
                metadata: serde_json::Value::Null,
            },
        ),
    )?;

    let retry_start = record_activity_started_idempotent(
        &ledger,
        ActivityStartedJournalRecord::new(
            ActivityJournalScope::run(run_id.clone()),
            5,
            activity_id.clone(),
            2,
        ),
    )?;

    assert_eq!(retry_start.status, ActivityJournalWriteStatus::Appended);
    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;
    assert_eq!(activity.status, ActivityStatus::Started);
    assert_eq!(activity.attempt, 2);

    Ok(())
}
