use std::error::Error;

use xiuxian_qianji_control::{
    Budget, ControlEventKind, ControlLedger, InMemoryControlLedger, RunAdmittedJournalRecord,
    RunCreatedJournalRecord, RunId, RunPlanRecordedJournalRecord, RunStatus,
    RunTerminalJournalRecord, record_run_admitted, record_run_created, record_run_plan_recorded,
    record_run_terminal,
};

#[test]
fn record_run_created_appends_replayable_run_fact() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-created-journal")?;
    let budget = Budget {
        wall_time_ms: Some(1_000),
        tokens: Some(2_000),
        cost_usd_micros: Some(3_000),
    };

    let record = record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "ship durable run", 11)
            .with_budget(budget.clone())
            .with_metadata(serde_json::json!({"source": "unit"})),
    )?;

    let ControlEventKind::RunCreated {
        intent,
        budget: recorded_budget,
        metadata,
    } = record.event.kind
    else {
        panic!("expected run-created event");
    };
    assert_eq!(intent, "ship durable run");
    assert_eq!(recorded_budget, Some(budget.clone()));
    assert_eq!(metadata["source"], "unit");

    let view = ledger.load_run_view(&run_id)?;
    assert_eq!(view.status, RunStatus::Draft);
    assert_eq!(view.intent.as_deref(), Some("ship durable run"));
    assert_eq!(view.budget, Some(budget));
    assert_eq!(view.updated_at_ms, 11);

    Ok(())
}

#[test]
fn run_created_record_builds_matching_event_without_appending() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("run-created-event-builder")?;

    let event = RunCreatedJournalRecord::new(run_id.clone(), "project workflow trace", 22)
        .with_metadata(serde_json::json!({"source": "trace"}))
        .into_event();

    assert_eq!(event.run_id, run_id);
    assert_eq!(event.occurred_at_ms, 22);
    let ControlEventKind::RunCreated {
        intent,
        budget,
        metadata,
    } = event.kind
    else {
        panic!("expected run-created event");
    };
    assert_eq!(intent, "project workflow trace");
    assert_eq!(budget, None);
    assert_eq!(metadata["source"], "trace");

    Ok(())
}

#[test]
fn run_lifecycle_journal_records_admission_and_plan() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-lifecycle-admission")?;

    record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "admit and plan", 10),
    )?;
    record_run_admitted(&ledger, RunAdmittedJournalRecord::new(run_id.clone(), 11))?;
    let plan_record = record_run_plan_recorded(
        &ledger,
        RunPlanRecordedJournalRecord::new(run_id.clone(), "Plan one durable slice", 12),
    )?;

    let ControlEventKind::PlanRecorded { summary } = plan_record.event.kind else {
        panic!("expected plan-recorded event");
    };
    assert_eq!(summary, "Plan one durable slice");

    let view = ledger.load_run_view(&run_id)?;
    assert_eq!(view.status, RunStatus::Planned);
    assert_eq!(view.updated_at_ms, 12);
    Ok(())
}

#[test]
fn run_lifecycle_journal_records_terminal_statuses() -> Result<(), Box<dyn Error>> {
    let completed = RunTerminalJournalRecord::completed(RunId::new("run-lifecycle-completed")?, 21)
        .into_event();
    assert!(matches!(completed.kind, ControlEventKind::RunCompleted));

    let failed =
        RunTerminalJournalRecord::failed(RunId::new("run-lifecycle-failed")?, "failed", 22)
            .into_event();
    assert!(matches!(
        failed.kind,
        ControlEventKind::RunFailed { message } if message == "failed"
    ));

    let blocked =
        RunTerminalJournalRecord::blocked(RunId::new("run-lifecycle-blocked")?, "blocked", 23)
            .into_event();
    assert!(matches!(
        blocked.kind,
        ControlEventKind::RunBlocked { reason } if reason == "blocked"
    ));

    let aborted =
        RunTerminalJournalRecord::aborted(RunId::new("run-lifecycle-aborted")?, "aborted", 24)
            .into_event();
    assert!(matches!(
        aborted.kind,
        ControlEventKind::RunAborted { reason } if reason == "aborted"
    ));

    Ok(())
}

#[test]
fn run_lifecycle_journal_records_terminal_fact_to_ledger() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-lifecycle-terminal")?;

    record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "finish run", 30),
    )?;
    record_run_admitted(&ledger, RunAdmittedJournalRecord::new(run_id.clone(), 31))?;
    record_run_terminal(
        &ledger,
        RunTerminalJournalRecord::completed(run_id.clone(), 32),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.updated_at_ms, 32);
    Ok(())
}
