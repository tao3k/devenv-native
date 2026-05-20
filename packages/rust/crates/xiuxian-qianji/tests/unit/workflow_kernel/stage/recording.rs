use crate::workflow_kernel::tests::support::{LenStage, TestContext, WorkflowRun, assert_err};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowControlRecordingFailure};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus,
};

#[tokio::test]
async fn workflow_kernel_finish_does_not_record_control_by_default() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow.no_control_default");
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let report = run.finish(output);

    assert_eq!(report.output, 4);
    assert!(
        ledger
            .load_events(
                &RunId::new("test.workflow.no_control_default")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_with_control_recording_is_opt_in() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow.control_recorded");
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let report = run
        .finish_with_control_recording(output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;
    let run_id = RunId::new("test.workflow.control_recorded").map_err(|error| error.to_string())?;
    let view = ledger
        .load_run_view(&run_id)
        .map_err(|error| error.to_string())?;

    assert_eq!(report.workflow.output, 4);
    assert_eq!(report.control.run_id, run_id);
    assert_eq!(report.control.terminal_status, RunStatus::Completed);
    assert_eq!(report.control.appended_event_count, 8);
    assert_eq!(report.control.run_view.status, RunStatus::Completed);
    assert_eq!(report.control.run_view.steps.len(), 1);
    assert_eq!(view.status, RunStatus::Completed);
    assert_eq!(view.steps.len(), 1);
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_with_recoverable_control_recording_records_on_clean_ledger()
-> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow.recoverable_recorded");
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let report = run
        .finish_with_recoverable_control_recording(output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    assert_eq!(report.workflow.output, 4);
    assert_eq!(report.control.terminal_status, RunStatus::Completed);
    assert_eq!(report.control.run_view.status, RunStatus::Completed);
    assert_eq!(report.control.run_view.steps.len(), 1);
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_recoverable_control_recording_preserves_report_on_control_error()
-> Result<(), String> {
    let ledger = InMemoryControlLedger::new();
    let mut context = TestContext::default();
    let mut first_run = WorkflowRun::new("test.workflow.recoverable_duplicate");
    let first_output = first_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    first_run
        .finish_with_control_recording(first_output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    let mut second_run = WorkflowRun::new("test.workflow.recoverable_duplicate");
    let second_output = second_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error: WorkflowControlRecordingFailure<usize> = assert_err(
        second_run.finish_with_recoverable_control_recording(
            second_output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "duplicate run should return the workflow report with the control error",
    );

    assert_eq!(error.workflow.output, 4);
    assert_eq!(
        error.workflow.trace.workflow_id,
        "test.workflow.recoverable_duplicate"
    );
    assert_eq!(error.workflow.trace.stages.len(), 1);
    assert!(matches!(
        error.source,
        ControlError::InvalidEventSequence { .. }
    ));
    assert_eq!(
        ledger
            .load_events(
                &RunId::new("test.workflow.recoverable_duplicate")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        8
    );
    Ok(())
}
