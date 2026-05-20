use crate::workflow_kernel::tests::support::{
    AppendStage, FailingStage, LenStage, TestContext, WorkflowRun, WorkflowStageStatus, assert_err,
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
