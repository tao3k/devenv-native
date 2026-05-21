use crate::workflow_kernel::tests::support::{
    LenStage, TestContext, WorkflowCompletionError, WorkflowRun, WorkflowTopology, assert_err,
};
use crate::workflow_kernel::{
    WorkflowCheckedControlRecordingError, WorkflowCheckedControlRecordingFailure,
    WorkflowControlRecorder, WorkflowControlRecordingPolicy,
};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus,
};

#[tokio::test]
async fn workflow_kernel_finish_checked_with_control_recording_validates_then_records()
-> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow.checked_control_recorded", ["len"])
        .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let report = run
        .finish_checked_with_control_recording(output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    assert_eq!(report.workflow.output, 4);
    assert_eq!(report.control.terminal_status, RunStatus::Completed);
    assert_eq!(report.control.run_view.status, RunStatus::Completed);
    assert_eq!(report.control.run_view.steps.len(), 1);
    assert_eq!(
        ledger
            .load_events(
                &RunId::new("test.workflow.checked_control_recorded")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        report.control.appended_event_count
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_checked_with_control_recording_rejects_incomplete_topology()
-> Result<(), String> {
    let topology = WorkflowTopology::linear(
        "test.workflow.checked_control_rejected",
        ["len", "append_b"],
    )
    .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        run.finish_checked_with_control_recording(output, WorkflowControlRecorder::new(&ledger)),
        "missing required stage should reject before control recording",
    );

    assert!(matches!(
        error,
        WorkflowCheckedControlRecordingError::Completion(
            WorkflowCompletionError::MissingRequiredStages { .. }
        )
    ));
    assert!(
        ledger
            .load_events(
                &RunId::new("test.workflow.checked_control_rejected")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_checked_with_recoverable_control_recording_validates_then_records()
-> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow.checked_recoverable_recorded", ["len"])
        .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let report = run
        .finish_checked_with_recoverable_control_recording(
            output,
            WorkflowControlRecorder::new(&ledger),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(report.workflow.output, 4);
    assert_eq!(report.control.terminal_status, RunStatus::Completed);
    assert_eq!(report.control.run_view.status, RunStatus::Completed);
    assert_eq!(report.control.run_view.steps.len(), 1);
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_checked_with_recoverable_control_recording_rejects_topology_before_append()
-> Result<(), String> {
    let topology = WorkflowTopology::linear(
        "test.workflow.checked_recoverable_rejected",
        ["len", "append_b"],
    )
    .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();

    let output = run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        run.finish_checked_with_recoverable_control_recording(
            output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "missing required stage should reject before recoverable control recording",
    );

    match error {
        WorkflowCheckedControlRecordingFailure::Completion { source } => assert!(matches!(
            *source,
            WorkflowCompletionError::MissingRequiredStages { .. }
        )),
        WorkflowCheckedControlRecordingFailure::Control { .. } => {
            panic!("expected completion failure before control recording")
        }
    }
    assert!(
        ledger
            .load_events(
                &RunId::new("test.workflow.checked_recoverable_rejected")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .is_empty()
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_checked_recoverable_control_recording_preserves_report_on_control_error()
-> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow.checked_recoverable_duplicate", ["len"])
        .map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();
    let mut context = TestContext::default();
    let mut first_run =
        WorkflowRun::new_with_topology(topology.clone()).map_err(|error| error.to_string())?;
    let first_output = first_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    first_run
        .finish_checked_with_control_recording(first_output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    let mut second_run =
        WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let second_output = second_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        second_run.finish_checked_with_recoverable_control_recording(
            second_output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "duplicate run should preserve the workflow report after validation",
    );

    match error {
        WorkflowCheckedControlRecordingFailure::Control { failure } => {
            assert_eq!(failure.workflow.output, 4);
            assert_eq!(
                failure.workflow.trace.workflow_id,
                "test.workflow.checked_recoverable_duplicate"
            );
            assert_eq!(failure.workflow.trace.stages.len(), 1);
            assert!(matches!(
                failure.source,
                ControlError::InvalidEventSequence { .. }
            ));
        }
        WorkflowCheckedControlRecordingFailure::Completion { .. } => {
            panic!("expected control recording failure after topology validation")
        }
    }
    assert_eq!(
        ledger
            .load_events(
                &RunId::new("test.workflow.checked_recoverable_duplicate")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        8
    );
    Ok(())
}

#[tokio::test]
async fn workflow_kernel_checked_recoverable_control_recording_retries_after_validation()
-> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow.checked_recoverable_retry", ["len"])
        .map_err(|error| error.to_string())?;
    let ledger = InMemoryControlLedger::new();
    let mut context = TestContext::default();
    let mut first_run =
        WorkflowRun::new_with_topology(topology.clone()).map_err(|error| error.to_string())?;
    let first_output = first_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    first_run
        .finish_checked_with_control_recording(first_output, WorkflowControlRecorder::new(&ledger))
        .map_err(|error| error.to_string())?;

    let mut second_run =
        WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let second_output = second_run
        .run_stage(&mut context, &LenStage, "seed".to_owned())
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        second_run.finish_checked_with_recoverable_control_recording(
            second_output,
            WorkflowControlRecorder::new(&ledger),
        ),
        "duplicate checked run should preserve the workflow report before retry",
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
                &RunId::new("test.workflow.checked_recoverable_retry")
                    .map_err(|error| error.to_string())?
            )
            .map_err(|error| error.to_string())?
            .len(),
        16
    );
    Ok(())
}
