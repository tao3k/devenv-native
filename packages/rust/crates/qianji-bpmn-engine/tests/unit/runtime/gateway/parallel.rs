use crate::runtime::{
    StubHost, parallel_dual_host_block_process, parallel_host_block_process, parallel_join_process,
    parallel_join_same_edge_duplicate_process,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, EventPollOutcome, PendingHostWorkKind,
    PendingHostWorkResult, ServiceTaskOutcome, advance_instance, apply_event_poll_outcome,
    apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_gateway_split_join_completes_deterministically() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_join_process("parallel_complete")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_complete",
        BpmnInstanceInit::new("wf_parallel_complete", json!({}), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(71))
        .await
        .must("bounded parallel gateway runtime should complete");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert!(instance.joins.is_empty());
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_gateway_surfaces_multiple_host_blocked_tokens() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process("parallel_dual_block")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_dual_block",
        BpmnInstanceInit::new("wf_parallel_dual_block", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(88);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel split should block on both service branches");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 2);
    assert_eq!(instance.pending_host_work, pending);
    assert_eq!(
        pending
            .iter()
            .map(|entry| entry.node_index)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(
        pending
            .iter()
            .map(|entry| entry.kind.clone())
            .collect::<Vec<_>>(),
        vec![PendingHostWorkKind::Service, PendingHostWorkKind::Service]
    );
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    );
    assert_eq!(instance.active_tokens.len(), 2);
    assert_eq!(
        instance
            .active_tokens
            .iter()
            .map(|token| token.node_index)
            .collect::<Vec<_>>(),
        vec![2, 3]
    );
    assert_eq!(
        pending
            .iter()
            .map(|entry| entry.token_id)
            .collect::<Vec<_>>(),
        instance
            .active_tokens
            .iter()
            .map(|token| token.token_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_gateway_preserves_join_arrival_while_host_branch_blocks() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_host_block_process("parallel_block")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_block",
        BpmnInstanceInit::new("wf_parallel_block", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(88);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel split should keep the join arrival while the service branch blocks");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Waiting
    );
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].node_index, 2);
    assert_eq!(pending[0].kind, PendingHostWorkKind::Service);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.active_tokens[0].token_id, pending[0].token_id);
    assert_eq!(instance.joins.len(), 1);
    assert_eq!(instance.joins[0].node_index, 4);
    assert_eq!(instance.joins[0].arrived, 1);
    assert_eq!(instance.joins[0].expected, 2);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );

    let resume = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome { data: json!({}) }),
        99,
    )
    .must("host completion should be applied");
    assert_eq!(resume, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(instance.joins.len(), 1);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("completed host branch should satisfy the outstanding join");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
    assert!(instance.active_tokens.is_empty());
    assert!(instance.joins.is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_gateway_same_edge_duplicates_do_not_fire_join_early() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_join_same_edge_duplicate_process(
            "parallel_same_edge_duplicate",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_same_edge_duplicate",
        BpmnInstanceInit::new("wf_parallel_same_edge_duplicate", json!({}), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(144);

    let waiting = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("same-edge duplicate arrivals should wait for the peer branch");
    assert_eq!(waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 4);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(instance.joins.len(), 1);
    assert_eq!(instance.joins[0].node_index, 5);
    assert_eq!(instance.joins[0].arrived, 2);
    assert_eq!(instance.joins[0].expected, 2);
    assert_eq!(instance.joins[0].incoming_counts, vec![2, 0]);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: Some(4),
            data: json!({}),
        },
        155,
    )
    .must("peer arrival should resume the waiting branch");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 5);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("join should fire once and preserve the extra buffered arrival");
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
