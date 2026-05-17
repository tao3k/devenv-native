use super::support::{
    AppendStage, TestContext, WorkflowCompletionError, WorkflowDuplicateStage, WorkflowEdgeKind,
    WorkflowRun, WorkflowStageBinding, WorkflowStageId, WorkflowStageStatus, WorkflowTopology,
    WorkflowTopologyEdge, WorkflowTopologyError, assert_err,
};

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
    let error = assert_err(
        WorkflowTopology::new(
            "test.workflow",
            vec![
                WorkflowStageBinding::required("extract"),
                WorkflowStageBinding::optional("extract"),
            ],
            Vec::new(),
        )
        .validate(),
        "duplicate stage id should be rejected",
    );

    assert_eq!(
        error,
        WorkflowTopologyError::DuplicateStage(WorkflowDuplicateStage {
            workflow_id: "test.workflow".into(),
            stage_id: WorkflowStageId::new("extract"),
        })
    );
}

#[test]
fn workflow_topology_rejects_empty_stage_list() {
    let error = assert_err(
        WorkflowTopology::new("test.workflow", Vec::new(), Vec::new()).validate(),
        "empty topology should be rejected",
    );

    assert_eq!(
        error,
        WorkflowTopologyError::EmptyStages {
            workflow_id: "test.workflow".to_owned(),
        }
    );
}

#[test]
fn workflow_topology_rejects_cycles() {
    let error = assert_err(
        WorkflowTopology::new(
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
        .validate(),
        "cycle should be rejected",
    );

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
    let error = assert_err(
        run.run_stage(
            &mut context,
            &AppendStage {
                id: "append_b",
                suffix: "-b",
            },
            "seed".to_owned(),
        )
        .await,
        "undeclared stage should be rejected before execution",
    );

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
    let error = assert_err(
        run.finish_checked(output),
        "missing required stage should be rejected",
    );

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
    let error = assert_err(
        run.finish_checked(output),
        "out-of-order edge completion should be rejected",
    );

    assert!(matches!(
        error,
        WorkflowCompletionError::EdgeOrderViolation { .. }
    ));

    Ok(())
}
