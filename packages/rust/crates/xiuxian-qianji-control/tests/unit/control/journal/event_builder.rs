use std::error::Error;

use xiuxian_qianji_control::{
    ActivityCompletedJournalRecord, ActivityFailedJournalRecord, ActivityFailure, ActivityId,
    ActivityJournalScope, ActivityResult, ActivityStartedJournalRecord,
    AdmittedActivityTaskScheduleRecord, ControlEventKind, ErrorCode, LeaseId, RecoveryItemScope,
    RunId, RunnableStep, SignalName, SignalReceiveJournalRecord, SignalRecord, StepId, StepLease,
    StepLeaseReleaseJournalRecord, StepQueueJournalRecord, TimerFireJournalRecord, TimerId,
    WorkerHeartbeat, WorkerHeartbeatJournalRecord, WorkerId,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn journal_record_into_event_preserves_scope_and_payloads() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("journal-record-builder-run")?;
    let step_id = StepId::new("journal-record-builder-step")?;

    let queued_event = StepQueueJournalRecord::new(
        RunnableStep {
            run_id: run_id.clone(),
            step_id: step_id.clone(),
            priority: 9,
            not_before_ms: 10,
            metadata: serde_json::Value::Null,
        },
        11,
    )
    .into_event();
    assert_eq!(queued_event.run_id, run_id);
    assert_eq!(queued_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(queued_event.kind, ControlEventKind::StepQueued));

    let lease = StepLease {
        lease_id: LeaseId::new("journal-record-builder-lease")?,
        run_id: queued_event.run_id.clone(),
        step_id: step_id.clone(),
        worker_id: WorkerId::new("journal-record-builder-worker")?,
        acquired_at_ms: 12,
        expires_at_ms: 24,
    };
    let lease_event = StepLeaseReleaseJournalRecord::new(lease.clone(), 25).into_event();
    assert_eq!(lease_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        lease_event.kind,
        ControlEventKind::StepLeaseReleased { lease: recorded } if recorded == lease
    ));

    let signal_event = SignalReceiveJournalRecord::new(
        queued_event.run_id.clone(),
        RecoveryItemScope::step(step_id.clone()),
        SignalRecord {
            signal_name: SignalName::new("journal.signal")?,
            payload_ref: None,
            payload_hash: Some("sha256:signal".to_owned()),
            metadata: serde_json::Value::Null,
        },
        30,
    )
    .into_event();
    assert_eq!(signal_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        signal_event.kind,
        ControlEventKind::SignalReceived { signal } if signal.payload_hash.as_deref() == Some("sha256:signal")
    ));

    let timer_id = TimerId::new("journal-record-builder-timer")?;
    let timer_event = TimerFireJournalRecord::new(
        queued_event.run_id.clone(),
        RecoveryItemScope::step(step_id.clone()),
        timer_id.clone(),
        40,
    )
    .into_event();
    assert_eq!(timer_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        timer_event.kind,
        ControlEventKind::TimerFired { timer_id: recorded } if recorded == timer_id
    ));

    let heartbeat_event = WorkerHeartbeatJournalRecord::new(
        queued_event.run_id,
        WorkerHeartbeat {
            worker_id: WorkerId::new("journal-record-builder-heartbeat")?,
            observed_at_ms: 50,
            expires_at_ms: 75,
            metadata: serde_json::Value::Null,
        },
    )
    .into_event();
    assert_eq!(heartbeat_event.occurred_at_ms, 50);
    assert!(matches!(
        heartbeat_event.kind,
        ControlEventKind::WorkerHeartbeatObserved { heartbeat }
            if heartbeat.worker_id.as_str() == "journal-record-builder-heartbeat"
    ));

    Ok(())
}

#[test]
fn activity_journal_record_into_event_preserves_scope_and_payloads() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("activity-journal-builder-run")?;
    let step_id = StepId::new("activity-journal-builder-step")?;
    let activity_id = ActivityId::new("activity-journal-builder-activity")?;
    let scope = ActivityJournalScope::step(run_id.clone(), step_id.clone());

    let scheduled_event = AdmittedActivityTaskScheduleRecord::step(
        run_id.clone(),
        step_id.clone(),
        10,
        activity_task(activity_id.clone())?,
    )
    .into_event();
    assert_eq!(scheduled_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        scheduled_event.kind,
        ControlEventKind::ActivityScheduled { task } if task.activity_id == activity_id
    ));

    let started_event =
        ActivityStartedJournalRecord::new(scope.clone(), 11, activity_id.clone(), 2)
            .with_worker_id(WorkerId::new("activity-journal-builder-worker")?)
            .into_event();
    assert_eq!(started_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        started_event.kind,
        ControlEventKind::ActivityStarted { activity_id: recorded, attempt, .. }
            if recorded == activity_id && attempt == 2
    ));

    let completed_event = ActivityCompletedJournalRecord::new(
        scope.clone(),
        12,
        activity_id.clone(),
        ActivityResult {
            output_ref: Some(artifact_ref("activity-journal-builder-output")?),
            output_hash: Some("sha256:activity-output".to_owned()),
            metadata: serde_json::Value::Null,
        },
    )
    .into_event();
    assert_eq!(completed_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        completed_event.kind,
        ControlEventKind::ActivityCompleted { activity_id: recorded, result }
            if recorded == activity_id && result.output_hash.as_deref() == Some("sha256:activity-output")
    ));

    let failed_event = ActivityFailedJournalRecord::new(
        scope,
        13,
        activity_id.clone(),
        ActivityFailure {
            error_code: ErrorCode::new("activity_failed")?,
            message: "activity failed during builder test".to_owned(),
            retryable: true,
            attempt: 2,
            metadata: serde_json::Value::Null,
        },
    )
    .into_event();
    assert_eq!(failed_event.step_id.as_ref(), Some(&step_id));
    assert!(matches!(
        failed_event.kind,
        ControlEventKind::ActivityFailed { activity_id: recorded, failure }
            if recorded == activity_id && failure.retryable
    ));

    Ok(())
}
