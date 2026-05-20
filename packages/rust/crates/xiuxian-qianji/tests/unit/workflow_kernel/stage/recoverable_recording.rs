use crate::workflow_kernel::tests::support::{LenStage, TestContext, WorkflowRun, assert_err};
use crate::workflow_kernel::{WorkflowControlRecorder, WorkflowControlRecordingPolicy};
use xiuxian_qianji_control::{ControlLedger, InMemoryControlLedger, RunId, RunStatus};

#[tokio::test]
async fn workflow_kernel_recoverable_control_recording_retries_from_retained_report()
-> Result<(), String> {
    let ledger = InMemoryControlLedger::new();
    let mut context = TestContext::default();
    let mut first_run = WorkflowRun::new("test.workflow.recoverable_retry");
    let first_output = first_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    first_run
        .finish_with_control_recording(first_output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    let mut second_run = WorkflowRun::new("test.workflow.recoverable_retry");
    let second_output = second_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        second_run.finish_with_recoverable_control_recording(
            second_output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "duplicate run should preserve the workflow report before retry",
    );
    let retry_report = error
        .retry_control_recording(
            WorkflowControlRecorder::new(&ledger)
                .with_policy(WorkflowControlRecordingPolicy::AppendOnly),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(context.events, vec!["len", "len"]);
    assert_eq!(retry_report.workflow.output, 4);
    assert_eq!(retry_report.control.appended_event_count, 8);
    assert_eq!(
        ledger
            .load_events(
                &RunId::new("test.workflow.recoverable_retry").map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        16
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_recoverable_control_recording_reuses_existing_run_without_rerun()
-> Result<(), String> {
    let ledger = InMemoryControlLedger::new();
    let mut context = TestContext::default();
    let mut first_run = WorkflowRun::new("test.workflow.recoverable_reuse");
    let first_output = first_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    first_run
        .finish_with_control_recording(first_output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    let mut second_run = WorkflowRun::new("test.workflow.recoverable_reuse");
    let second_output = second_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        second_run.finish_with_recoverable_control_recording(
            second_output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "duplicate run should preserve the workflow report before reuse",
    );
    let retry_report = error
        .retry_control_recording(
            WorkflowControlRecorder::new(&ledger)
                .with_policy(WorkflowControlRecordingPolicy::ReuseExistingRun),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(context.events, vec!["len", "len"]);
    assert_eq!(retry_report.workflow.output, 4);
    assert_eq!(retry_report.control.appended_event_count, 0);
    assert!(retry_report.control.records.is_empty());
    assert_eq!(retry_report.control.run_view.status, RunStatus::Completed);
    assert_eq!(
        ledger
            .load_events(
                &RunId::new("test.workflow.recoverable_reuse").map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        8
    );
    Ok(())
}
