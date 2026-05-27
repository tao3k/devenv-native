use serde_json::json;
use xiuxian_qianji_control::{
    ControlError, ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId,
    StepStatus, WorkflowTraceProjectionRecord, WorkflowTraceProjectionStage,
    WorkflowTraceProjectionStageInput, record_workflow_trace_projection,
};

#[test]
fn workflow_trace_projection_records_successful_run() -> Result<(), ControlError> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("workflow.projection")?;
    let projection = WorkflowTraceProjectionRecord::new(run_id.clone(), "workflow:projection", 100)
        .with_metadata(json!({"source": "test", "stageCount": 1}))
        .with_plan_summary("Workflow trace with 1 stage(s)")
        .with_stages(vec![
            WorkflowTraceProjectionStage::succeeded(
                WorkflowTraceProjectionStageInput::new(
                    StepId::new("load")?,
                    "load",
                    "workflow_kernel_stage",
                )
                .with_timestamps(100, 104)
                .with_metadata(json!({"stageId": "load"})),
            )
            .with_required_evidence(vec!["validation_path".to_owned()]),
        ]);

    let records = record_workflow_trace_projection(&ledger, projection)?;
    let view = ledger.load_run_view(&run_id)?;

    assert_eq!(records.len(), 8);
    assert!(matches!(
        records[0].event.kind,
        ControlEventKind::RunCreated { .. }
    ));
    assert_eq!(view.status, RunStatus::Completed);
    let Some(step) = view.steps.get(&StepId::new("load")?) else {
        panic!("expected load step");
    };
    assert_eq!(step.status, StepStatus::Succeeded);
    assert_eq!(step.required_evidence, vec!["validation_path".to_owned()]);
    Ok(())
}

#[test]
fn workflow_trace_projection_records_first_failed_stage_as_run_failure() -> Result<(), ControlError>
{
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("workflow.projection.failed")?;
    let projection =
        WorkflowTraceProjectionRecord::new(run_id.clone(), "workflow:projection.failed", 200)
            .with_metadata(json!({"source": "test", "stageCount": 1}))
            .with_plan_summary("Workflow trace with 1 stage(s)")
            .with_stages(vec![WorkflowTraceProjectionStage::failed(
                WorkflowTraceProjectionStageInput::new(
                    StepId::new("parse")?,
                    "parse",
                    "workflow_kernel_stage",
                )
                .with_timestamps(200, 205)
                .with_metadata(json!({"stageId": "parse"})),
                "parser rejected input",
            )]);

    let records = record_workflow_trace_projection(&ledger, projection)?;
    let view = ledger.load_run_view(&run_id)?;

    assert_eq!(records.len(), 8);
    assert_eq!(view.status, RunStatus::Failed);
    let Some(step) = view.steps.get(&StepId::new("parse")?) else {
        panic!("expected parse step");
    };
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    Ok(())
}

#[test]
fn workflow_trace_projection_rejects_blank_required_evidence() -> Result<(), ControlError> {
    let projection = WorkflowTraceProjectionRecord::new(
        RunId::new("workflow.projection.invalid")?,
        "workflow:projection.invalid",
        300,
    )
    .with_metadata(json!({"source": "test", "stageCount": 1}))
    .with_plan_summary("Workflow trace with 1 stage(s)")
    .with_stages(vec![
        WorkflowTraceProjectionStage::succeeded(
            WorkflowTraceProjectionStageInput::new(
                StepId::new("validate")?,
                "validate",
                "workflow_kernel_stage",
            )
            .with_timestamps(300, 301)
            .with_metadata(json!({"stageId": "validate"})),
        )
        .with_required_evidence(vec![" ".to_owned()]),
    ]);

    assert!(matches!(
        projection.into_events(),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    Ok(())
}
