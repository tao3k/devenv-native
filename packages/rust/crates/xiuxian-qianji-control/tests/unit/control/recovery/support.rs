use std::error::Error;

use xiuxian_qianji_control::{
    ActivityFailure, ControlEvent, ControlEventKind, ControlLedger, ErrorCode,
    InMemoryControlLedger, LeaseId, RunId, StepId, StepLease, TimerId, TimerRecord, WorkerId,
};

pub(crate) fn append_run_created(
    ledger: &InMemoryControlLedger,
    run_id: RunId,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::run(
        run_id,
        1,
        ControlEventKind::RunCreated {
            intent: "derive replay recovery state".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    Ok(())
}

pub(crate) fn append_step_created(
    ledger: &InMemoryControlLedger,
    run_id: RunId,
    step_id: StepId,
    occurred_at_ms: u64,
) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::step(
        run_id,
        step_id,
        occurred_at_ms,
        ControlEventKind::StepCreated {
            title: "Recovery step".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    Ok(())
}

pub(crate) fn activity_failure(
    error_code: &str,
    retryable: bool,
    attempt: u32,
) -> Result<ActivityFailure, Box<dyn Error>> {
    Ok(ActivityFailure {
        error_code: ErrorCode::new(error_code)?,
        message: format!("activity failed with {error_code}"),
        retryable,
        attempt,
        metadata: serde_json::Value::Null,
    })
}

pub(crate) const fn timer_record(timer_id: TimerId, fire_at_ms: u64) -> TimerRecord {
    TimerRecord {
        timer_id,
        fire_at_ms,
        metadata: serde_json::Value::Null,
    }
}

pub(crate) fn step_lease(
    run_id: &RunId,
    step_id: &StepId,
    lease_id: &str,
    acquired_at_ms: u64,
    expires_at_ms: u64,
) -> Result<StepLease, Box<dyn Error>> {
    Ok(StepLease {
        lease_id: LeaseId::new(lease_id)?,
        run_id: run_id.clone(),
        step_id: step_id.clone(),
        worker_id: WorkerId::new(format!("worker-{lease_id}"))?,
        acquired_at_ms,
        expires_at_ms,
    })
}
