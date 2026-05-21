use super::support::{
    Arc, AtomicUsize, Duration, Ordering, WorkflowBoundedFanoutStageRequest, WorkflowRun,
    WorkflowStageFacts, WorkflowStageId, WorkflowStageStatus, WorkflowTopology, assert_err,
};

#[tokio::test]
async fn workflow_kernel_bounded_fanout_preserves_input_order() -> Result<(), String> {
    let mut run = WorkflowRun::new("test.workflow");

    let outputs = run
        .run_bounded_fanout_stage(WorkflowBoundedFanoutStageRequest {
            stage_id: WorkflowStageId::new("fanout"),
            inputs: vec![30_u64, 10, 20],
            max_concurrency: 3,
            input_facts: WorkflowStageFacts::typed("Vec<u64>").with_item_count(3),
            output_facts: |outputs: &[(usize, u64)]| {
                WorkflowStageFacts::typed("Vec<u64>").with_item_count(outputs.len())
            },
            operation: |index, delay_ms| async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                Ok::<_, String>((index, delay_ms))
            },
        })
        .await
        .map_err(|error| error.to_string())?;
    let report = run.finish(outputs);

    assert_eq!(report.output, vec![(0, 30), (1, 10), (2, 20)]);
    assert_eq!(report.trace.stages.len(), 1);
    assert_eq!(report.trace.stages[0].stage_id, "fanout");
    assert_eq!(
        report.trace.stages[0].status,
        WorkflowStageStatus::Succeeded
    );
    assert_eq!(report.trace.stages[0].input.item_count, Some(3));
    assert_eq!(report.trace.stages[0].output.item_count, Some(3));

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_bounded_fanout_respects_concurrency_cap() -> Result<(), String> {
    let active = Arc::new(AtomicUsize::new(0));
    let max_active = Arc::new(AtomicUsize::new(0));
    let mut run = WorkflowRun::new("test.workflow");

    let outputs = run
        .run_bounded_fanout_stage(WorkflowBoundedFanoutStageRequest {
            stage_id: WorkflowStageId::new("fanout"),
            inputs: vec![1_usize, 2, 3, 4],
            max_concurrency: 2,
            input_facts: WorkflowStageFacts::typed("Vec<usize>").with_item_count(4),
            output_facts: |outputs: &[usize]| {
                WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len())
            },
            operation: {
                let active = Arc::clone(&active);
                let max_active = Arc::clone(&max_active);
                move |_index, input| {
                    let active = Arc::clone(&active);
                    let max_active = Arc::clone(&max_active);
                    async move {
                        let now_active = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max_active.fetch_max(now_active, Ordering::SeqCst);
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        active.fetch_sub(1, Ordering::SeqCst);
                        Ok::<_, String>(input * 2)
                    }
                }
            },
        })
        .await
        .map_err(|error| error.to_string())?;

    assert_eq!(outputs, vec![2, 4, 6, 8]);
    assert!(max_active.load(Ordering::SeqCst) <= 2);

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_bounded_fanout_records_failed_item_index() -> Result<(), String> {
    let mut run = WorkflowRun::new("test.workflow");
    let error = assert_err(
        run.run_bounded_fanout_stage(WorkflowBoundedFanoutStageRequest {
            stage_id: WorkflowStageId::new("fanout"),
            inputs: vec![1_usize, 2, 3],
            max_concurrency: 2,
            input_facts: WorkflowStageFacts::typed("Vec<usize>").with_item_count(3),
            output_facts: |outputs: &[usize]| {
                WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len())
            },
            operation: |index, input| async move {
                if index == 1 {
                    Err("bad shard".to_owned())
                } else {
                    Ok(input)
                }
            },
        })
        .await,
        "failing item should fail the fan-out stage",
    );

    assert_eq!(error.stage_id, "fanout");
    assert!(error.message.contains("fan-out item `1` failed"));
    assert_eq!(error.trace.stages.len(), 1);
    assert_eq!(error.trace.stages[0].status, WorkflowStageStatus::Failed);

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_bounded_fanout_rejects_undeclared_topology_stage() -> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow", ["declared"])
        .map_err(|error| error.to_string())?;
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let error = assert_err(
        run.run_bounded_fanout_stage(WorkflowBoundedFanoutStageRequest {
            stage_id: WorkflowStageId::new("fanout"),
            inputs: vec![1_usize],
            max_concurrency: 1,
            input_facts: WorkflowStageFacts::typed("Vec<usize>").with_item_count(1),
            output_facts: |outputs: &[usize]| {
                WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len())
            },
            operation: |_index, input| async move { Ok::<_, String>(input) },
        })
        .await,
        "undeclared fan-out stage should be rejected",
    );

    assert_eq!(error.stage_id, "fanout");
    assert!(error.message.contains("not declared by workflow topology"));

    Ok(())
}
