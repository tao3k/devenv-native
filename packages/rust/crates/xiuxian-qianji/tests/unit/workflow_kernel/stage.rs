use super::support::{
    AppendStage, FailingStage, LenStage, TestContext, WorkflowCheckedControlRecordingError,
    WorkflowCheckedControlRecordingFailure, WorkflowCompletionError, WorkflowControlRecorder,
    WorkflowControlRecordingFailure, WorkflowControlRecordingPolicy, WorkflowEdgeKind, WorkflowRun,
    WorkflowStageFacts, WorkflowStageStatus, WorkflowTopology, assert_err,
};
use xiuxian_qianji_control::{
    ControlError, ControlLedger, InMemoryControlLedger, RunId, RunStatus,
};

#[tokio::test]
async fn workflow_kernel_records_typed_stage_order_and_report() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow");

    let first = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_a",
                suffix: "-a",
            },
            "seed".to_owned(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let second = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_b",
                suffix: "-b",
            },
            first,
        )
        .await
        .map_err(|error| error.to_string())?;
    let output = run
        .run_stage(&mut context, &LenStage, second)
        .await
        .map_err(|error| error.to_string())?;
    let report = run.finish(output);

    assert_eq!(report.output, 8);
    assert_eq!(context.events, vec!["append_a", "append_b", "len"]);
    assert_eq!(
        report
            .trace
            .stages
            .iter()
            .map(|stage| stage.stage_id.as_str())
            .collect::<Vec<_>>(),
        vec!["append_a", "append_b", "len"]
    );
    assert!(
        report
            .trace
            .stages
            .iter()
            .all(|stage| stage.status == WorkflowStageStatus::Succeeded)
    );
    assert_eq!(report.trace.stages[0].input.item_count, Some(4));
    assert_eq!(report.trace.stages[2].output.item_count, Some(8));

    Ok(())
}

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

#[tokio::test]
async fn workflow_kernel_short_circuits_with_failed_stage_trace() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow");

    let first = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_a",
                suffix: "-a",
            },
            "seed".to_owned(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let error = assert_err(
        run.run_stage(&mut context, &FailingStage, first).await,
        "failing stage should return a workflow execution error",
    );

    assert_eq!(error.workflow_id, "test.workflow");
    assert_eq!(error.stage_id, "fail");
    assert_eq!(error.message, "intentional failure");
    assert_eq!(context.events, vec!["append_a", "fail"]);
    assert_eq!(error.trace.stages.len(), 2);
    assert_eq!(error.trace.stages[1].status, WorkflowStageStatus::Failed);
    assert_eq!(
        error.trace.stages[1].error.as_deref(),
        Some("intentional failure")
    );

    Ok(())
}

#[test]
fn workflow_kernel_facts_describe_arrow_edges_without_owning_arrow_buffers() {
    let facts = WorkflowStageFacts::arrow_record_batch("xiuxian_wendao.audio_shard_input", "v1")
        .with_item_count(12)
        .with_cache_hit_count(3);

    assert_eq!(facts.item_count, Some(12));
    assert_eq!(facts.cache_hit_count, Some(3));
    assert_eq!(
        facts.edge_kind,
        Some(WorkflowEdgeKind::ArrowRecordBatch {
            schema_name: "xiuxian_wendao.audio_shard_input".to_owned(),
            schema_version: "v1".to_owned(),
        })
    );
}
