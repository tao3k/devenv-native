use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use super::{
    WorkflowCheckpointError, WorkflowCheckpointStorageKind, WorkflowCompletionError,
    WorkflowEdgeKind, WorkflowRun, WorkflowStage, WorkflowStageBinding, WorkflowStageFacts,
    WorkflowStageStatus, WorkflowTopology, WorkflowTopologyEdge, WorkflowTopologyError,
};

#[derive(Debug, Default)]
struct TestContext {
    events: Vec<&'static str>,
}

#[derive(Debug)]
struct AppendStage {
    id: &'static str,
    suffix: &'static str,
}

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for AppendStage {
    type Output = String;
    type Error = String;

    fn id(&self) -> &'static str {
        self.id
    }

    fn input_facts(&self, input: &String) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(input.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(output.len())
    }

    async fn run(&self, context: &mut TestContext, input: String) -> Result<Self::Output, String> {
        context.events.push(self.id);
        Ok(format!("{input}{}", self.suffix))
    }
}

#[derive(Debug)]
struct LenStage;

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for LenStage {
    type Output = usize;
    type Error = String;

    fn id(&self) -> &'static str {
        "len"
    }

    fn input_facts(&self, input: &String) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("String").with_item_count(input.len())
    }

    fn output_facts(&self, output: &Self::Output) -> WorkflowStageFacts {
        WorkflowStageFacts::typed("usize").with_item_count(*output)
    }

    async fn run(&self, context: &mut TestContext, input: String) -> Result<Self::Output, String> {
        context.events.push("len");
        Ok(input.len())
    }
}

#[derive(Debug)]
struct FailingStage;

#[async_trait::async_trait]
impl WorkflowStage<TestContext, String> for FailingStage {
    type Output = String;
    type Error = String;

    fn id(&self) -> &'static str {
        "fail"
    }

    async fn run(&self, context: &mut TestContext, _input: String) -> Result<Self::Output, String> {
        context.events.push("fail");
        Err("intentional failure".to_owned())
    }
}

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
    let error = run
        .run_stage(&mut context, &FailingStage, first)
        .await
        .expect_err("failing stage should return a workflow execution error");

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
        .record_memory_checkpoint(
            "append_a",
            "checkpoint.append_a.output",
            WorkflowStageFacts::typed("String").with_item_count(output.len()),
            Some("sha256:test".to_owned()),
            Arc::new(output.clone()),
        )
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
        .get::<String>("checkpoint.append_a.output")
        .map_err(|error| error.to_string())?;
    assert_eq!(payload.as_str(), "seed-a");

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_rejects_memory_checkpoint_for_unsuccessful_stage() {
    let mut run = WorkflowRun::new("test.workflow");
    let error = run
        .record_memory_checkpoint(
            "append_a",
            "checkpoint.append_a.output",
            WorkflowStageFacts::typed("String"),
            None,
            Arc::new("seed".to_owned()),
        )
        .expect_err("checkpoint should require a successful stage trace");

    assert_eq!(
        error,
        WorkflowCheckpointError::StageNotSucceeded {
            stage_id: "append_a".to_owned(),
            checkpoint_id: "checkpoint.append_a.output".to_owned(),
        }
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
    run.record_memory_checkpoint(
        "append_a",
        "checkpoint.append_a.output",
        WorkflowStageFacts::typed("String").with_item_count(output.len()),
        None,
        Arc::new(output.clone()),
    )
    .map_err(|error| error.to_string())?;
    let report = run.finish(output);
    let error = report
        .memory_checkpoints
        .get::<usize>("checkpoint.append_a.output")
        .expect_err("wrong payload type should be rejected");

    assert!(matches!(
        error,
        WorkflowCheckpointError::PayloadTypeMismatch { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_bounded_fanout_preserves_input_order() -> Result<(), String> {
    let mut run = WorkflowRun::new("test.workflow");

    let outputs = run
        .run_bounded_fanout_stage(
            "fanout",
            vec![30_u64, 10, 20],
            3,
            WorkflowStageFacts::typed("Vec<u64>").with_item_count(3),
            |outputs| WorkflowStageFacts::typed("Vec<u64>").with_item_count(outputs.len()),
            |index, delay_ms| async move {
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                Ok::<_, String>((index, delay_ms))
            },
        )
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
        .run_bounded_fanout_stage(
            "fanout",
            vec![1_usize, 2, 3, 4],
            2,
            WorkflowStageFacts::typed("Vec<usize>").with_item_count(4),
            |outputs| WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len()),
            {
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
        )
        .await
        .map_err(|error| error.to_string())?;

    assert_eq!(outputs, vec![2, 4, 6, 8]);
    assert!(max_active.load(Ordering::SeqCst) <= 2);

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_bounded_fanout_records_failed_item_index() -> Result<(), String> {
    let mut run = WorkflowRun::new("test.workflow");
    let error = run
        .run_bounded_fanout_stage(
            "fanout",
            vec![1_usize, 2, 3],
            2,
            WorkflowStageFacts::typed("Vec<usize>").with_item_count(3),
            |outputs| WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len()),
            |index, input| async move {
                if index == 1 {
                    Err("bad shard".to_owned())
                } else {
                    Ok(input)
                }
            },
        )
        .await
        .expect_err("failing item should fail the fan-out stage");

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
    let error = run
        .run_bounded_fanout_stage(
            "fanout",
            vec![1_usize],
            1,
            WorkflowStageFacts::typed("Vec<usize>").with_item_count(1),
            |outputs| WorkflowStageFacts::typed("Vec<usize>").with_item_count(outputs.len()),
            |_index, input| async move { Ok::<_, String>(input) },
        )
        .await
        .expect_err("undeclared fan-out stage should be rejected");

    assert_eq!(error.stage_id, "fanout");
    assert!(error.message.contains("not declared by workflow topology"));

    Ok(())
}

#[test]
fn workflow_topology_orders_dependency_edges() -> Result<(), String> {
    let topology = WorkflowTopology::new(
        "test.workflow",
        vec![
            WorkflowStageBinding::required("extract"),
            WorkflowStageBinding::required("merge"),
            WorkflowStageBinding::required("validate"),
        ],
        vec![
            WorkflowTopologyEdge::with_edge_kind(
                "extract",
                "merge",
                WorkflowEdgeKind::arrow_record_batch("xiuxian_wendao.audio_shard_result", "v1"),
            ),
            WorkflowTopologyEdge::new("merge", "validate"),
        ],
    );

    assert_eq!(
        topology
            .topological_stage_ids()
            .map_err(|error| error.to_string())?,
        vec!["extract", "merge", "validate"]
    );

    Ok(())
}

#[test]
fn workflow_topology_rejects_duplicate_stage_ids() {
    let error = WorkflowTopology::new(
        "test.workflow",
        vec![
            WorkflowStageBinding::required("extract"),
            WorkflowStageBinding::optional("extract"),
        ],
        Vec::new(),
    )
    .validate()
    .expect_err("duplicate stage id should be rejected");

    assert_eq!(
        error,
        WorkflowTopologyError::DuplicateStage {
            workflow_id: "test.workflow".to_owned(),
            stage_id: "extract".to_owned(),
        }
    );
}

#[test]
fn workflow_topology_rejects_empty_stage_list() {
    let error = WorkflowTopology::new("test.workflow", Vec::new(), Vec::new())
        .validate()
        .expect_err("empty topology should be rejected");

    assert_eq!(
        error,
        WorkflowTopologyError::EmptyStages {
            workflow_id: "test.workflow".to_owned(),
        }
    );
}

#[test]
fn workflow_topology_rejects_cycles() {
    let error = WorkflowTopology::new(
        "test.workflow",
        vec![
            WorkflowStageBinding::required("extract"),
            WorkflowStageBinding::required("merge"),
        ],
        vec![
            WorkflowTopologyEdge::new("extract", "merge"),
            WorkflowTopologyEdge::new("merge", "extract"),
        ],
    )
    .validate()
    .expect_err("cycle should be rejected");

    assert_eq!(
        error,
        WorkflowTopologyError::Cycle {
            workflow_id: "test.workflow".to_owned(),
        }
    );
}

#[tokio::test]
async fn workflow_kernel_rejects_undeclared_topology_stage() -> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow", ["append_a"])
        .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let error = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_b",
                suffix: "-b",
            },
            "seed".to_owned(),
        )
        .await
        .expect_err("undeclared stage should be rejected before execution");

    assert_eq!(context.events, Vec::<&'static str>::new());
    assert_eq!(error.stage_id, "append_b");
    assert_eq!(error.trace.stages.len(), 1);
    assert_eq!(error.trace.stages[0].status, WorkflowStageStatus::Failed);
    assert!(
        error
            .message
            .contains("is not declared by workflow topology")
    );

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_checked_rejects_missing_required_stage() -> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow", ["append_a", "append_b"])
        .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
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
    let error = run
        .finish_checked(output)
        .expect_err("missing required stage should be rejected");

    assert!(matches!(
        error,
        WorkflowCompletionError::MissingRequiredStages { .. }
    ));
    if let WorkflowCompletionError::MissingRequiredStages {
        missing_stage_ids, ..
    } = error
    {
        assert_eq!(missing_stage_ids, vec!["append_b"]);
    }

    Ok(())
}

#[tokio::test]
async fn workflow_kernel_finish_checked_rejects_edge_order_violation() -> Result<(), String> {
    let topology = WorkflowTopology::linear("test.workflow", ["append_a", "append_b"])
        .map_err(|error| error.to_string())?;
    let mut context = TestContext::default();
    let mut run = WorkflowRun::new_with_topology(topology).map_err(|error| error.to_string())?;
    let second = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_b",
                suffix: "-b",
            },
            "seed".to_owned(),
        )
        .await
        .map_err(|error| error.to_string())?;
    let output = run
        .run_stage(
            &mut context,
            &AppendStage {
                id: "append_a",
                suffix: "-a",
            },
            second,
        )
        .await
        .map_err(|error| error.to_string())?;
    let error = run
        .finish_checked(output)
        .expect_err("out-of-order edge completion should be rejected");

    assert!(matches!(
        error,
        WorkflowCompletionError::EdgeOrderViolation { .. }
    ));

    Ok(())
}
