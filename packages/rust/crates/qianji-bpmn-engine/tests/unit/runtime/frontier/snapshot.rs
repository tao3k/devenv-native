use super::super::{StubHost, parallel_dual_host_block_process, parallel_host_and_wait_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnFrontierEntryStatus, BpmnInstanceInit, BpmnPackage, advance_instance,
    create_instance, snapshot_frontier,
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
