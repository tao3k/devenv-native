use std::error::Error;

use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, InMemoryControlLedger, RunCreatedJournalRecord, RunId,
    RunStatus, StepCreatedJournalRecord, StepId, StepStartedJournalRecord, StepStatus,
    StepTerminalJournalRecord, StepToolCallJournalRecord, record_run_created, record_step_created,
    record_step_started, record_step_terminal, record_step_tool_call,
};

#[test]
fn step_lifecycle_journal_records_create_start_and_tool_call() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("step-lifecycle")?;
    let step_id = StepId::new("load")?;

    record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "step lifecycle", 10),
    )?;
    record_step_created(
        &ledger,
        StepCreatedJournalRecord::new(run_id.clone(), step_id.clone(), "Load", 11)
            .with_required_evidence(vec!["authority".to_owned()]),
    )?;
    record_step_started(
        &ledger,
        StepStartedJournalRecord::new(run_id.clone(), step_id.clone(), 12),
    )?;
    let tool_call = record_step_tool_call(
        &ledger,
        StepToolCallJournalRecord::new(run_id.clone(), step_id.clone(), "workflow_stage", 13)
            .with_metadata(serde_json::json!({"stageId": "load"})),
    )?;

    let ControlEventKind::ToolCallRecorded {
        tool_name,
        metadata,
    } = tool_call.event.kind
    else {
        panic!("expected tool-call event");
    };
    assert_eq!(tool_name, "workflow_stage");
    assert_eq!(metadata["stageId"], "load");

    let view = ledger.load_run_view(&run_id)?;
    assert_eq!(view.status, RunStatus::Draft);
    let Some(step) = view.steps.get(&step_id) else {
        panic!("expected load step");
    };
    assert_eq!(step.status, StepStatus::Running);
    assert_eq!(step.required_evidence, vec!["authority".to_owned()]);
    assert_eq!(view.updated_at_ms, 13);
    Ok(())
}

#[test]
fn step_lifecycle_journal_records_terminal_statuses() -> Result<(), Box<dyn Error>> {
    let succeeded = StepTerminalJournalRecord::succeeded(
        RunId::new("step-lifecycle-succeeded")?,
        StepId::new("succeeded")?,
        21,
    )
    .into_event();
    assert!(matches!(succeeded.kind, ControlEventKind::StepSucceeded));

    let failed = StepTerminalJournalRecord::failed(
        RunId::new("step-lifecycle-failed")?,
        StepId::new("failed")?,
        "failed_code",
        "failed",
        false,
        22,
    )
    .into_event();
    assert!(matches!(
        failed.kind,
        ControlEventKind::StepFailed { error_code, message, retryable }
            if error_code == "failed_code" && message == "failed" && !retryable
    ));

    let blocked = StepTerminalJournalRecord::blocked(
        RunId::new("step-lifecycle-blocked")?,
        StepId::new("blocked")?,
        "blocked",
        23,
    )
    .into_event();
    assert!(matches!(
        blocked.kind,
        ControlEventKind::StepBlocked { reason } if reason == "blocked"
    ));

    let cancelled = StepTerminalJournalRecord::cancelled(
        RunId::new("step-lifecycle-cancelled")?,
        StepId::new("cancelled")?,
        "cancelled",
        24,
    )
    .into_event();
    assert!(matches!(
        cancelled.kind,
        ControlEventKind::StepCancelled { reason } if reason == "cancelled"
    ));
    Ok(())
}

#[test]
fn step_lifecycle_journal_records_terminal_fact_to_ledger() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("step-lifecycle-terminal")?;
    let step_id = StepId::new("audit")?;

    record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id.clone(), "finish step", 30),
    )?;
    record_step_created(
        &ledger,
        StepCreatedJournalRecord::new(run_id.clone(), step_id.clone(), "Audit", 31),
    )?;
    record_step_terminal(
        &ledger,
        StepTerminalJournalRecord::succeeded(run_id.clone(), step_id.clone(), 32),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let Some(step) = view.steps.get(&step_id) else {
        panic!("expected audit step");
    };
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(view.updated_at_ms, 32);
    Ok(())
}
