use crate::workflow_kernel::tests::control::support::{
    evidence_ref, record_workflow_stage_evidence, stage_trace,
};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowStageStatus, WorkflowTrace};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId, StepStatus,
};

#[test]
fn workflow_stage_evidence_replays_to_step_without_status_change() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.evidence".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            7_200,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let evidence = evidence_ref("evidence-validation-path", Some("validation_path"))?;
    let record = record_workflow_stage_evidence(
        &ledger,
        "workflow.evidence",
        "validate",
        7_250,
        evidence.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.evidence")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(record.sequence, 9);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.evidence, vec![evidence]);
    assert_eq!(
        step.covered_required_evidence(),
        vec!["validation_path".to_owned()]
    );
    Ok(())
}

#[test]
fn workflow_stage_evidence_rejects_blank_ids() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_stage_evidence(
            &ledger,
            " ",
            "validate",
            0,
            evidence_ref("evidence-validation-path", Some("validation_path"))?,
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_evidence(
            &ledger,
            "workflow.evidence",
            " ",
            0,
            evidence_ref("evidence-validation-path", Some("validation_path"))?,
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    Ok(())
}
