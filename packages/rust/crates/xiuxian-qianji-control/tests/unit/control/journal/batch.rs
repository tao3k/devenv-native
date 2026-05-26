use std::error::Error;

use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunAdmittedJournalRecord,
    RunCreatedJournalRecord, RunId, RunStatus, StepCreatedJournalRecord, StepId, StepStatus,
    record_control_event_batch,
};

#[test]
fn journal_batch_records_same_run_events_and_replays_view() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("journal-batch-run")?;
    let step_id = StepId::new("journal-batch-step")?;

    let outcome = record_control_event_batch(
        &ledger,
        vec![
            RunCreatedJournalRecord::new(run_id.clone(), "record batch", 10).into_event(),
            RunAdmittedJournalRecord::new(run_id.clone(), 11).into_event(),
            StepCreatedJournalRecord::new(run_id.clone(), step_id.clone(), "batch step", 12)
                .into_event(),
        ],
    )?;

    assert_eq!(outcome.run_id, run_id);
    assert_eq!(outcome.appended_event_count, 3);
    assert_eq!(outcome.records.len(), 3);
    assert_eq!(outcome.run_view.status, RunStatus::Admitted);
    let Some(step) = outcome.run_view.steps.get(&step_id) else {
        panic!("expected journal batch step");
    };
    assert_eq!(step.status, StepStatus::Pending);
    assert_eq!(step.title.as_deref(), Some("batch step"));
    Ok(())
}

#[test]
fn journal_batch_rejects_empty_event_batches() {
    let ledger = InMemoryControlLedger::new();
    let Err(error) = record_control_event_batch(&ledger, Vec::new()) else {
        panic!("empty journal batch should fail");
    };

    assert!(matches!(error, ControlError::InvalidEventSequence { .. }));
    assert!(error.to_string().contains("cannot be empty"));
}

#[test]
fn journal_batch_rejects_mixed_run_ids_before_append() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let first_run_id = RunId::new("journal-batch-first-run")?;
    let second_run_id = RunId::new("journal-batch-second-run")?;

    let Err(error) = record_control_event_batch(
        &ledger,
        vec![
            RunCreatedJournalRecord::new(first_run_id.clone(), "first", 10).into_event(),
            RunAdmittedJournalRecord::new(second_run_id.clone(), 11).into_event(),
        ],
    ) else {
        panic!("mixed-run journal batch should fail");
    };

    assert!(matches!(error, ControlError::InvalidEventSequence { .. }));
    assert!(error.to_string().contains("cannot mix run ids"));
    assert!(ledger.load_events(&first_run_id)?.is_empty());
    assert!(ledger.load_events(&second_run_id)?.is_empty());
    Ok(())
}
