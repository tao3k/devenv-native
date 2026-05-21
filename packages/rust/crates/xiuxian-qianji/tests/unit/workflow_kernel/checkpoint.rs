use super::support::{
    AppendStage, Arc, TestContext, WorkflowCheckpointError, WorkflowCheckpointId,
    WorkflowCheckpointStorageKind, WorkflowMemoryCheckpointRecord, WorkflowRun,
    WorkflowStageCheckpointMiss, WorkflowStageFacts, WorkflowStageId, assert_err,
};

#[tokio::test]
async fn workflow_kernel_records_memory_checkpoint_metadata_and_payload() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow");
    let output = run
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
    let checkpoint = run
        .record_memory_checkpoint(WorkflowMemoryCheckpointRecord {
            stage_id: WorkflowStageId::new("append_a"),
            checkpoint_id: WorkflowCheckpointId::new("checkpoint.append_a.output"),
            facts: WorkflowStageFacts::typed("String").with_item_count(output.len()),
            content_fingerprint: Some("sha256:test".to_owned()),
            payload: Arc::new(output.clone()),
        })
        .map_err(|error| error.to_string())?;
    let report = run.finish(output);

    assert_eq!(
        checkpoint.storage_kind,
        WorkflowCheckpointStorageKind::Memory
    );
    assert_eq!(checkpoint.stage_id, "append_a");
    assert_eq!(checkpoint.item_count, Some(6));
    assert_eq!(
        checkpoint.content_fingerprint.as_deref(),
        Some("sha256:test")
    );
    assert_eq!(
        report.trace.stages[0]
            .checkpoints
            .iter()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str())
            .collect::<Vec<_>>(),
        vec!["checkpoint.append_a.output"]
    );
    let payload = report
        .memory_checkpoints
        .get::<String>(&WorkflowCheckpointId::new("checkpoint.append_a.output"))
        .map_err(|error| error.to_string())?;
    assert_eq!(payload.as_str(), "seed-a");

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_rejects_memory_checkpoint_for_unsuccessful_stage() {
    let mut run = WorkflowRun::new("test.workflow");
    let error = assert_err(
        run.record_memory_checkpoint(WorkflowMemoryCheckpointRecord {
            stage_id: WorkflowStageId::new("append_a"),
            checkpoint_id: WorkflowCheckpointId::new("checkpoint.append_a.output"),
            facts: WorkflowStageFacts::typed("String"),
            content_fingerprint: None,
            payload: Arc::new("seed".to_owned()),
        }),
        "checkpoint should require a successful stage trace",
    );

    assert_eq!(
        error,
        WorkflowCheckpointError::StageNotSucceeded(WorkflowStageCheckpointMiss {
            stage_id: WorkflowStageId::new("append_a"),
            checkpoint_id: WorkflowCheckpointId::new("checkpoint.append_a.output"),
        })
    );
}

#[tokio::test]
async fn workflow_kernel_rejects_memory_checkpoint_type_mismatch() -> Result<(), String> {
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new("test.workflow");
    let output = run
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
    run.record_memory_checkpoint(WorkflowMemoryCheckpointRecord {
        stage_id: WorkflowStageId::new("append_a"),
        checkpoint_id: WorkflowCheckpointId::new("checkpoint.append_a.output"),
        facts: WorkflowStageFacts::typed("String").with_item_count(output.len()),
        content_fingerprint: None,
        payload: Arc::new(output.clone()),
    })
    .map_err(|error| error.to_string())?;
    let report = run.finish(output);
    let error = assert_err(
        report
            .memory_checkpoints
            .get::<usize>(&WorkflowCheckpointId::new("checkpoint.append_a.output")),
        "wrong payload type should be rejected",
    );

    assert!(matches!(
        error,
        WorkflowCheckpointError::PayloadTypeMismatch { .. }
    ));

    Ok(())
}
