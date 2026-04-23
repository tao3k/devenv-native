use super::super::parallel_dual_host_block_process;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnFrontierExecutionBatch, BpmnFrontierExecutionProposal, BpmnFrontierExecutionStep,
    BpmnFrontierPlanAction, BpmnInstanceInit, BpmnPackage, InstanceLifecycle, NodeRuntimeStatus,
    TokenRecord, collect_frontier_proposals, create_instance, reduce_frontier_plan,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn runtime_frontier_proposals_collect_all_runnable_tokens_in_order() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process(
            "parallel_runnable_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_runnable_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_runnable_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 3;
    instance.lifecycle = InstanceLifecycle::Running;
    instance.active_tokens = vec![
        TokenRecord {
            token_id: 7,
            node_index: 2,
            incoming_edge_index: Some(1),
            inclusive_join_hint: None,
        },
        TokenRecord {
            token_id: 8,
            node_index: 3,
            incoming_edge_index: Some(2),
            inclusive_join_hint: None,
        },
    ];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = NodeRuntimeStatus::Queued;
    instance.node_states[3].status = NodeRuntimeStatus::Queued;

    let proposals = collect_frontier_proposals(&instance);

    assert_eq!(proposals.snapshot.first_runnable_token_index(), Some(0));
    assert_eq!(
        proposals.execution_proposals,
        vec![
            BpmnFrontierExecutionProposal {
                token_id: 7,
                token_index: 0,
                node_index: 2,
                incoming_edge_index: Some(1),
            },
            BpmnFrontierExecutionProposal {
                token_id: 8,
                token_index: 1,
                node_index: 3,
                incoming_edge_index: Some(2),
            },
        ]
    );
}

#[test]
fn runtime_frontier_plan_reduces_runnable_proposals_into_batch_deterministically() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process(
            "parallel_reduce_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_reduce_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_reduce_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 3;
    instance.lifecycle = InstanceLifecycle::Running;
    instance.active_tokens = vec![
        TokenRecord {
            token_id: 7,
            node_index: 2,
            incoming_edge_index: Some(1),
            inclusive_join_hint: None,
        },
        TokenRecord {
            token_id: 8,
            node_index: 3,
            incoming_edge_index: Some(2),
            inclusive_join_hint: None,
        },
    ];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = NodeRuntimeStatus::Queued;
    instance.node_states[3].status = NodeRuntimeStatus::Queued;

    let proposals = collect_frontier_proposals(&instance);
    let plan = reduce_frontier_plan(&package.processes[0], &instance, proposals);

    assert_eq!(
        plan.proposals.snapshot.first_runnable_token_index(),
        Some(0)
    );
    assert_eq!(
        plan.action,
        BpmnFrontierPlanAction::ExecuteBatch(BpmnFrontierExecutionBatch {
            proposals: vec![
                BpmnFrontierExecutionProposal {
                    token_id: 7,
                    token_index: 0,
                    node_index: 2,
                    incoming_edge_index: Some(1),
                },
                BpmnFrontierExecutionProposal {
                    token_id: 8,
                    token_index: 1,
                    node_index: 3,
                    incoming_edge_index: Some(2),
                },
            ],
            steps: vec![
                BpmnFrontierExecutionStep::Proposal(BpmnFrontierExecutionProposal {
                    token_id: 7,
                    token_index: 0,
                    node_index: 2,
                    incoming_edge_index: Some(1),
                }),
                BpmnFrontierExecutionStep::Proposal(BpmnFrontierExecutionProposal {
                    token_id: 8,
                    token_index: 1,
                    node_index: 3,
                    incoming_edge_index: Some(2),
                }),
            ],
        })
    );
}
