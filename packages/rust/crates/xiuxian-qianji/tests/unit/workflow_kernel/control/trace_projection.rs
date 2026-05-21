use super::support::stage_trace;
use crate::workflow_kernel::{
    WorkflowControlEvidenceRequirements, WorkflowStageStatus, WorkflowTrace,
    record_workflow_trace_to_control_ledger,
    record_workflow_trace_to_control_ledger_with_required_evidence,
    workflow_trace_to_control_event_records,
    workflow_trace_to_control_event_records_with_required_evidence,
    workflow_trace_to_control_events,
};
use xiuxian_qianji_control::{
    ControlError, ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, RunStatus, StepId,
    StepStatus,
};

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
fn workflow_trace_required_evidence_projection_is_opt_in() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_000,
            1_000_000,
            None,
        )],
    };

    let default_view =
        xiuxian_qianji_control::replay_run_view(workflow_trace_to_control_event_records(&trace)?)?;
    let Some(default_step) = default_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };
    assert!(default_step.required_evidence.is_empty());

    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("validate", ["validation_path", "authority"])?;
    let required_view = xiuxian_qianji_control::replay_run_view(
        workflow_trace_to_control_event_records_with_required_evidence(&trace, &requirements)?,
    )?;
    let Some(required_step) = required_view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(
        required_step.required_evidence,
        vec!["validation_path".to_owned(), "authority".to_owned()]
    );
    Ok(())
}

#[test]
fn workflow_trace_required_evidence_records_to_injected_ledger() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence_ledger".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_100,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("validate", ["validation_path"])?;

    let records = record_workflow_trace_to_control_ledger_with_required_evidence(
        &ledger,
        &trace,
        &requirements,
    )?;
    let view = ledger.load_run_view(&RunId::new("workflow.required_evidence_ledger")?)?;
    let Some(step) = view.steps.get(&StepId::new("validate")?) else {
        panic!("expected validate step");
    };

    assert_eq!(records.len(), 8);
    assert_eq!(step.required_evidence, vec!["validation_path".to_owned()]);
    Ok(())
}

#[test]
fn workflow_trace_required_evidence_rejects_invalid_requirements() -> Result<(), ControlError> {
    assert!(matches!(
        WorkflowControlEvidenceRequirements::new().require_stage_evidence(" ", ["validation_path"]),
        Err(ControlError::BlankId { field: "step_id" })
    ));
    assert!(matches!(
        WorkflowControlEvidenceRequirements::new().require_stage_evidence("validate", [" "]),
        Err(ControlError::BlankId {
            field: "required_evidence"
        })
    ));

    let trace = WorkflowTrace {
        workflow_id: "workflow.required_evidence_invalid".to_owned(),
        stages: vec![stage_trace(
            "validate",
            WorkflowStageStatus::Succeeded,
            1_200,
            1_000_000,
            None,
        )],
    };
    let requirements = WorkflowControlEvidenceRequirements::new()
        .require_stage_evidence("missing", ["validation_path"])?;

    assert!(matches!(
        workflow_trace_to_control_event_records_with_required_evidence(&trace, &requirements),
        Err(ControlError::InvalidEventSequence { .. })
    ));
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
fn workflow_trace_records_to_injected_control_ledger() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.ledger".to_owned(),
        stages: vec![
            stage_trace(
                "collect",
                WorkflowStageStatus::Succeeded,
                3_000,
                1_000_000,
                None,
            ),
            stage_trace(
                "audit",
                WorkflowStageStatus::Succeeded,
                3_010,
                2_000_000,
                None,
            ),
        ],
    };
    let ledger = InMemoryControlLedger::new();

    let records = record_workflow_trace_to_control_ledger(&ledger, &trace)?;
    let view = ledger.load_run_view(&RunId::new("workflow.ledger")?)?;

    assert_eq!(records.len(), 12);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=12).collect::<Vec<_>>()
    );
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.steps.len(), 2);
    assert!(
        view.steps
            .values()
            .all(|step| step.status == StepStatus::Succeeded)
    );
    Ok(())
}
