use super::super::{StubHost, parallel_dual_host_block_process, parallel_host_and_wait_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnFrontierEntryStatus, BpmnInstanceInit, BpmnPackage,
    InstanceLifecycle, NodeRuntimeStatus, TokenRecord, WaitKind, WaitRegistration,
    advance_instance, create_instance, snapshot_frontier,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_frontier_snapshot_classifies_parallel_host_blocked_tokens() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process(
            "parallel_dual_block_frontier",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_dual_block_frontier",
        BpmnInstanceInit::new("wf_parallel_dual_block_frontier", json!({}), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(88))
        .await
        .must("parallel split should surface both blocked service branches");
    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let frontier = snapshot_frontier(&instance);
    assert_eq!(frontier.first_runnable_token_index(), None);
    assert_eq!(
        frontier
            .entries
            .iter()
            .map(|entry| (entry.token_index, entry.node_index, entry.status))
            .collect::<Vec<_>>(),
        vec![
            (0, 2, BpmnFrontierEntryStatus::BlockedOnHost),
            (1, 3, BpmnFrontierEntryStatus::BlockedOnHost),
        ]
    );
}

#[test]
fn runtime_frontier_snapshot_keeps_queued_boundary_owner_runnable() {
    let mut instance = boundary_owner_frontier_fixture(NodeRuntimeStatus::Queued);

    let frontier = snapshot_frontier(&instance);

    assert_eq!(frontier.first_runnable_token_index(), Some(0));
    assert_eq!(
        frontier.entries[0].status,
        BpmnFrontierEntryStatus::Runnable
    );
    instance.node_states[2].status = NodeRuntimeStatus::Executing;
    let waiting_frontier = snapshot_frontier(&instance);
    assert_eq!(waiting_frontier.first_runnable_token_index(), None);
    assert_eq!(
        waiting_frontier.entries[0].status,
        BpmnFrontierEntryStatus::WaitingExternal
    );
}

fn boundary_owner_frontier_fixture(
    owner_status: NodeRuntimeStatus,
) -> qianji_bpmn_engine::BpmnInstanceState {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_host_and_wait_process(
            "boundary_owner_frontier_classification",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_owner_frontier_classification",
        BpmnInstanceInit::new("wf_boundary_owner_frontier_classification", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 7;
    instance.lifecycle = InstanceLifecycle::Waiting;
    instance.active_tokens = vec![TokenRecord {
        token_id: 17,
        node_index: 2,
        incoming_edge_index: Some(1),
        inclusive_join_hint: None,
    }];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = owner_status;
    instance.waits.push(WaitRegistration {
        process_id: Some("boundary_owner_frontier_classification".to_string()),
        node_index: 3,
        blocking_node_index: Some(2),
        kind: WaitKind::Timer,
        event_kind: Some(BpmnEventKind::Timer),
        event_reference: Some("boundary_timeout".to_string()),
        event_name: Some("BoundaryTimeout".to_string()),
        timer: None,
        correlation_key: None,
    });
    instance
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_frontier_snapshot_classifies_mixed_host_and_wait_branches() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_host_and_wait_process(
            "parallel_host_wait_frontier",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_host_wait_frontier",
        BpmnInstanceInit::new("wf_parallel_host_wait_frontier", json!({}), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(88))
        .await
        .must("parallel split should keep both blocked frontier branches visible");
    assert!(matches!(outcome, BpmnAdvanceOutcome::BlockedOnHost(_)));

    let frontier = snapshot_frontier(&instance);
    assert_eq!(frontier.first_runnable_token_index(), None);
    assert_eq!(
        frontier
            .entries
            .iter()
            .map(|entry| (entry.token_index, entry.node_index, entry.status))
            .collect::<Vec<_>>(),
        vec![
            (0, 2, BpmnFrontierEntryStatus::BlockedOnHost),
            (1, 3, BpmnFrontierEntryStatus::WaitingExternal),
        ]
    );
}
