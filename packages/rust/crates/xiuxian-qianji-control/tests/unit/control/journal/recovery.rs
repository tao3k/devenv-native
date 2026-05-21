use std::error::Error;

use xiuxian_qianji_control::{
    ControlLedger, InMemoryControlLedger, RecoveryAttempt, RecoveryItemScope, RecoveryPolicy,
    RecoveryStartedJournalRecord, RunId, RunStatus, StepId, StepStatus, record_recovery_started,
};

#[test]
fn recovery_attempt_journal_records_run_scoped_recovery() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-recovery-attempt-journal")?;

    let record = record_recovery_started(
        &ledger,
        RecoveryStartedJournalRecord::new(
            run_id.clone(),
            RecoveryItemScope::run(),
            recovery_attempt(1, "recover run"),
            10,
        ),
    )?;
    let view = ledger.load_run_view(&run_id)?;

    assert_eq!(record.sequence, 1);
    assert_eq!(view.status, RunStatus::Recovering);
    Ok(())
}

#[test]
fn recovery_attempt_journal_records_step_scoped_recovery() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-step-recovery-attempt-journal")?;
    let step_id = StepId::new("stage-recover")?;

    record_recovery_started(
        &ledger,
        RecoveryStartedJournalRecord::new(
            run_id.clone(),
            RecoveryItemScope::step(step_id.clone()),
            recovery_attempt(2, "recover step"),
            10,
        ),
    )?;
    let view = ledger.load_run_view(&run_id)?;
    let step = view.steps.get(&step_id).ok_or("missing recovery step")?;

    assert_eq!(step.status, StepStatus::Recovering);
    assert_eq!(step.recovery_attempts.len(), 1);
    assert_eq!(step.recovery_attempts[0].reason, "recover step");
    Ok(())
}

fn recovery_attempt(attempt: u32, reason: &str) -> RecoveryAttempt {
    RecoveryAttempt {
        attempt,
        reason: reason.to_owned(),
        policy: RecoveryPolicy {
            max_attempts: 3,
            backoff_ms: 50,
            require_human_approval: false,
        },
    }
}
