use crate::workflow_kernel::{
    WorkflowStageFacts, WorkflowStageStatus, WorkflowStageTrace, WorkflowTrace,
    workflow_trace_to_control_event_records, workflow_trace_to_control_events,
};
use xiuxian_qianji_control::{ControlError, ControlEventKind, RunStatus, StepStatus};

#[test]
fn workflow_trace_maps_successful_stages_to_replayable_control_view() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.success".to_owned(),
        stages: vec![
            stage_trace(
                "load",
                WorkflowStageStatus::Succeeded,
                1_000,
                2_000_000,
                None,
            ),
            stage_trace(
                "render",
                WorkflowStageStatus::Succeeded,
                1_005,
                3_000_000,
                None,
            ),
        ],
    };

    let events = workflow_trace_to_control_events(&trace)?;
    assert_eq!(events.len(), 12);
    assert!(matches!(
        events[0].kind,
        ControlEventKind::RunCreated { .. }
    ));
    assert!(matches!(
        events[3].kind,
        ControlEventKind::StepCreated { .. }
    ));
    assert!(matches!(
        events[5].kind,
        ControlEventKind::ToolCallRecorded { .. }
    ));

    let view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.intent.as_deref(), Some("workflow:workflow.success"));
    assert_eq!(view.steps.len(), 2);
    assert!(
        view.steps
            .values()
            .all(|step| step.status == StepStatus::Succeeded)
    );
    assert_eq!(view.updated_at_ms, 1_008);
    Ok(())
}

#[test]
fn workflow_trace_maps_failed_stage_to_failed_control_view() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.failed".to_owned(),
        stages: vec![stage_trace(
            "parse",
            WorkflowStageStatus::Failed,
            2_000,
            4_000_000,
            Some("parser rejected input"),
        )],
    };

    let view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;

    assert_eq!(view.status, RunStatus::Failed);
    assert_eq!(view.steps.len(), 1);
    let Some(step) = view.steps.values().next() else {
        panic!("expected one failed step");
    };
    assert_eq!(step.status, StepStatus::Failed);
    assert_eq!(step.last_error.as_deref(), Some("parser rejected input"));
    Ok(())
}

#[test]
fn workflow_trace_rejects_blank_control_ids() {
    let trace = WorkflowTrace {
        workflow_id: " ".to_owned(),
        stages: Vec::new(),
    };

    assert!(matches!(
        workflow_trace_to_control_events(&trace),
        Err(ControlError::BlankId { field: "run_id" })
    ));
}

fn stage_trace(
    stage_id: &str,
    status: WorkflowStageStatus,
    started_unix_ms: u64,
    duration_nanos: u64,
    error: Option<&str>,
) -> WorkflowStageTrace {
    WorkflowStageTrace {
        stage_id: stage_id.to_owned(),
        status,
        started_unix_ms,
        duration_nanos,
        input: WorkflowStageFacts::typed("input").with_item_count(1),
        output: WorkflowStageFacts::typed("output").with_item_count(1),
        error: error.map(str::to_owned),
        checkpoints: Vec::new(),
    }
}
