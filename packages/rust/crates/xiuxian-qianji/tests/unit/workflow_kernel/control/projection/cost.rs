use crate::workflow_kernel::tests::control::support::{
    cost_observation, record_workflow_run_cost_observation, record_workflow_stage_cost_observation,
    stage_trace,
};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowStageStatus, WorkflowTrace};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId, StepStatus,
};

#[test]
fn workflow_cost_observations_replay_to_run_and_step_totals() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.cost".to_owned(),
        stages: vec![stage_trace(
            "infer",
            WorkflowStageStatus::Succeeded,
            7_300,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let run_observation = cost_observation("planner", 120, 30, 200);
    let step_observation = cost_observation("llm", 900, 180, 1_700);
    let run_record = record_workflow_run_cost_observation(
        &ledger,
        "workflow.cost",
        7_350,
        run_observation.clone(),
    )?;
    let step_record = record_workflow_stage_cost_observation(
        &ledger,
        "workflow.cost",
        "infer",
        7_360,
        step_observation.clone(),
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.cost")?)?;
    let Some(step) = view.steps.get(&StepId::new("infer")?) else {
        panic!("expected infer step");
    };

    assert_eq!(run_record.sequence, 9);
    assert_eq!(step_record.sequence, 10);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(view.cost_observations, vec![run_observation]);
    assert_eq!(step.cost_observations, vec![step_observation]);
    assert_eq!(step.total_cost_usd_micros(), 1_700);
    assert_eq!(view.total_cost_usd_micros(), 1_900);
    Ok(())
}

#[test]
fn workflow_cost_observations_reject_blank_ids() {
    let ledger = InMemoryControlLedger::new();

    assert!(matches!(
        record_workflow_run_cost_observation(&ledger, " ", 0, cost_observation("planner", 1, 1, 1)),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_cost_observation(
            &ledger,
            " ",
            "infer",
            0,
            cost_observation("llm", 1, 1, 1),
        ),
        Err(ControlError::BlankId { field: "run_id" })
    ));
    assert!(matches!(
        record_workflow_stage_cost_observation(
            &ledger,
            "workflow.cost",
            " ",
            0,
            cost_observation("llm", 1, 1, 1),
        ),
        Err(ControlError::BlankId { field: "step_id" })
    ));
}
