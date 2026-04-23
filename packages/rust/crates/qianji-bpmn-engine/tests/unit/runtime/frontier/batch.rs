use super::super::{StubHost, parallel_join_process, parallel_join_same_edge_duplicate_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal,
    BpmnFrontierExecutionStep, BpmnFrontierParallelJoinMerge, BpmnFrontierPlanAction,
    BpmnInstanceInit, BpmnInstanceState, BpmnPackage, InstanceLifecycle, NodeRuntimeStatus,
    PendingHostWorkKind, TokenRecord, advance_instance, create_instance, plan_frontier_step,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_frontier_batch_execution_reindexes_parallel_join_tokens_deterministically() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_join_process("parallel_join_frontier_batch")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_join_frontier_batch",
        BpmnInstanceInit::new("wf_parallel_join_frontier_batch", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 3;
    instance.lifecycle = InstanceLifecycle::Running;
    instance.active_tokens = vec![
        TokenRecord {
            token_id: 7,
            node_index: 4,
            incoming_edge_index: Some(3),
            inclusive_join_hint: None,
        },
        TokenRecord {
            token_id: 8,
            node_index: 4,
            incoming_edge_index: Some(4),
            inclusive_join_hint: None,
        },
    ];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = NodeRuntimeStatus::Completed;
    instance.node_states[3].status = NodeRuntimeStatus::Completed;
    instance.node_states[4].status = NodeRuntimeStatus::Queued;

    let plan = plan_frontier_step(&package.processes[0], &instance);
    assert_eq!(
        plan.action,
        parallel_join_action(
            4,
            vec![
                frontier_proposal(7, 0, 4, Some(3)),
                frontier_proposal(8, 1, 4, Some(4)),
            ],
        )
    );

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(71))
        .await
        .must("parallel join batch should re-resolve token ownership after in-batch removal");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert!(instance.joins.is_empty());
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[5].status, NodeRuntimeStatus::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_frontier_parallel_join_merge_preserves_excess_buffered_arrivals() {
    let (package, mut instance) = merged_parallel_join_duplicate_fixture();

    let plan = plan_frontier_step(&package.processes[0], &instance);
    assert_eq!(
        plan.action,
        parallel_join_action(
            5,
            vec![
                frontier_proposal(9, 0, 5, Some(5)),
                frontier_proposal(10, 1, 5, Some(5)),
                frontier_proposal(11, 2, 5, Some(6)),
            ],
        )
    );

    let blocked = advance_instance(package.as_ref(), &mut instance, &StubHost::new(155))
        .await
        .must("merged join arrivals should fire once and preserve the excess duplicate");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].node_index, 6);
    assert_eq!(pending[0].kind, PendingHostWorkKind::Service);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 6);
    assert_eq!(instance.joins.len(), 1);
    assert_eq!(instance.joins[0].node_index, 5);
    assert_eq!(instance.joins[0].arrived, 1);
    assert_eq!(instance.joins[0].expected, 2);
    assert_eq!(instance.joins[0].incoming_counts, vec![1, 0]);
}

fn frontier_proposal(
    token_id: u64,
    token_index: usize,
    node_index: u32,
    incoming_edge_index: Option<u32>,
) -> BpmnFrontierExecutionProposal {
    BpmnFrontierExecutionProposal {
        token_id,
        token_index,
        node_index,
        incoming_edge_index,
    }
}

fn parallel_join_action(
    node_index: u32,
    proposals: Vec<BpmnFrontierExecutionProposal>,
) -> BpmnFrontierPlanAction {
    BpmnFrontierPlanAction::ExecuteBatch(BpmnFrontierExecutionBatch {
        steps: vec![BpmnFrontierExecutionStep::ParallelJoin(
            BpmnFrontierParallelJoinMerge {
                node_index,
                proposals: proposals.clone(),
            },
        )],
        proposals,
    })
}

fn merged_parallel_join_duplicate_fixture() -> (Arc<BpmnPackage>, BpmnInstanceState) {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_join_same_edge_duplicate_process(
            "parallel_join_frontier_merge_duplicates",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_join_frontier_merge_duplicates",
        BpmnInstanceInit::new("wf_parallel_join_frontier_merge_duplicates", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 5;
    instance.lifecycle = InstanceLifecycle::Running;
    instance.active_tokens = vec![
        TokenRecord {
            token_id: 9,
            node_index: 5,
            incoming_edge_index: Some(5),
            inclusive_join_hint: None,
        },
        TokenRecord {
            token_id: 10,
            node_index: 5,
            incoming_edge_index: Some(5),
            inclusive_join_hint: None,
        },
        TokenRecord {
            token_id: 11,
            node_index: 5,
            incoming_edge_index: Some(6),
            inclusive_join_hint: None,
        },
    ];
    for node_index in 0..=4 {
        instance.node_states[node_index].status = NodeRuntimeStatus::Completed;
    }
    instance.node_states[5].status = NodeRuntimeStatus::Queued;
    (package, instance)
}
