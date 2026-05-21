use crate::workflow_kernel::tests::control::support::{
    gate_result, record_workflow_stage_gate_result, stage_trace,
};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowStageStatus, WorkflowTrace};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId, StepStatus,
};

#[test]
fn workflow_stage_gate_result_replays_to_step_without_status_change() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.gate".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            7_400,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let result = gate_result("required-evidence", true)?;
    let record = record_workflow_stage_gate_result(
        &ledger,
        "workflow.gate",
        "validate",
        7_450,
        result.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.gate")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.gate_results, vec![result]);
    Ok(())
}

#[test]
fn workflow_stage_gate_result_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_gate_result(
            &ledger,
            " ",
            "validate",
            0,
            gate_result("required-evidence", false)?,
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_gate_result(
            &ledger,
            "workflow.gate",
            " ",
            0,
            gate_result("required-evidence", false)?,
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}
