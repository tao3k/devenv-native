use super::support::stage_trace;
use crate::workflow_kernel::{
    WorkflowControlRecorder, WorkflowControlRecordingPolicy, WorkflowStageStatus, WorkflowTrace,
    workflow_trace_to_control_events,
};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus,
};

#[test]
fn workflow_control_recorder_rejects_existing_run_by_default() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.duplicate".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            4_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let first = WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let second = WorkflowControlRecorder::new(&ledger).record_trace(&trace);

    assert_eq!(first.run_id.as_str(), "workflow.duplicate");
    assert_eq!(first.terminal_status, RunStatus::Completed);
    assert_eq!(first.appended_event_count, 8);
    assert_eq!(first.run_view.status, RunStatus::Completed);
    assert_eq!(first.run_view.steps.len(), 1);
    assert!(matches!(
        second,
        Err(ControlError::InvalidEventSequence { .. })
    ));
    assert_eq!(
        ledger
            .load_events(&RunId::new("workflow.duplicate")?)?
            .len(),
        8
    );
    Ok(())
}

#[test]
fn workflow_control_recorder_supports_explicit_append_only_mode() -> Result<(), ControlError> {
    let trace = WorkflowTrace {
        workflow_id: "workflow.append_only".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            5_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();
    let recorder = WorkflowControlRecorder::new(&ledger)
        .with_policy(WorkflowControlRecordingPolicy::AppendOnly);

    let first = recorder.record_trace(&trace)?;
    let second = recorder.record_trace(&trace)?;
    let records = ledger.load_events(&RunId::new("workflow.append_only")?)?;

    assert_eq!(first.appended_event_count, 8);
    assert_eq!(second.appended_event_count, 8);
    assert_eq!(second.run_view.status, RunStatus::Completed);
    assert_eq!(second.run_view.steps.len(), 1);
    assert_eq!(records.len(), 16);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=16).collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn workflow_control_recorder_can_reuse_existing_run_without_appending() -> Result<(), ControlError>
{
    let trace = WorkflowTrace {
        workflow_id: "workflow.reuse_existing".to_owned(),
        stages: vec![stage_trace(
            "collect",
            WorkflowStageStatus::Succeeded,
            6_000,
            1_000_000,
            None,
        )],
    };
    let ledger = InMemoryControlLedger::new();

    let first = WorkflowControlRecorder::new(&ledger).record_trace(&trace)?;
    let second = WorkflowControlRecorder::new(&ledger)
        .with_policy(WorkflowControlRecordingPolicy::ReuseExistingRun)
        .record_trace(&trace)?;
    let records = ledger.load_events(&RunId::new("workflow.reuse_existing")?)?;

    assert_eq!(first.appended_event_count, 8);
    assert_eq!(second.appended_event_count, 0);
    assert!(second.records.is_empty());
    assert_eq!(second.run_view.status, RunStatus::Completed);
    assert_eq!(second.run_view.steps.len(), 1);
    assert_eq!(records.len(), 8);
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        (1..=8).collect::<Vec<_>>()
    );
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
