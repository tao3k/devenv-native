use crate::workflow_kernel::tests::control::support::{
    record_workflow_run_recovery_attempt, record_workflow_stage_recovery_attempt, recovery_attempt,
    stage_trace,
};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowStageStatus, WorkflowTrace};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId, StepStatus,
};

#[test]
fn workflow_stage_recovery_attempt_replays_to_recovering_step() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.recovery".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            7_000,
            1_000_000,
            Some("parser rejected input"),
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let attempt = recovery_attempt(1);
    let record = record_workflow_stage_recovery_attempt(
        &ledger,
        "workflow.recovery",
        "parse",
        7_050,
        attempt.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.recovery")?)?;
    let Some(step) = view.steps.get(&StepId::new("parse")?) else {
        panic!("expected parse step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Failed);
    assert_eq!(step.status, StepStatus::Recovering);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    assert_eq!(step.recovery_attempts, vec![attempt]);
    Ok(())
}

#[test]
fn workflow_run_recovery_attempt_replays_to_recovering_run() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.run_recovery".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            7_100,
            1_000_000,
            Some("parser rejected input"),
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let attempt = recovery_attempt(1);
    let record = record_workflow_run_recovery_attempt(
        &ledger,
        "workflow.run_recovery",
        7_150,
        attempt.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.run_recovery")?)?;
    let Some(step) = view.steps.get(&StepId::new("parse")?) else {
        panic!("expected parse step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Recovering);
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    assert!(step.recovery_attempts.is_empty());
    Ok(())
}

#[test]
fn workflow_stage_recovery_attempt_rejects_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_recovery_attempt(&ledger, " ", "parse", 0, recovery_attempt(1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_recovery_attempt(
            &ledger,
            "workflow.recovery",
            " ",
            0,
            recovery_attempt(1),
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
}

#[test]
fn workflow_run_recovery_attempt_rejects_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_run_recovery_attempt(&ledger, " ", 0, recovery_attempt(1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
}
