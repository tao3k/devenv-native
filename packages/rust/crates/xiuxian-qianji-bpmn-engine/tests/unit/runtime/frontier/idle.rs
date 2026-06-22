use crate::runtime::{StubHost, parallel_dual_host_block_process, parallel_host_and_wait_process};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnFrontierPlanAction, BpmnInstanceInit, BpmnPackage,
    InstanceLifecycle, NodeRuntimeStatus, SuspendReason, TokenRecord, WaitKind, WaitRegistration,
    advance_instance, create_instance, plan_frontier_step,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_frontier_plan_returns_blocked_on_host_idle_outcome() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process(
            "parallel_dual_block_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_dual_block_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_dual_block_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(88))
        .await
        .must("parallel split should block on both service branches");
    let pending = match outcome {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    let plan = plan_frontier_step(&package.processes[0], &instance);

    assert_eq!(plan.proposals.snapshot.first_runnable_token_index(), None);
    assert!(plan.proposals.execution_proposals.is_empty());
    assert_eq!(plan.action, BpmnFrontierPlanAction::BlockedOnHost(pending));
}

#[test]
fn runtime_frontier_plan_returns_waiting_idle_outcome() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_host_and_wait_process(
            "parallel_wait_only_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_wait_only_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_wait_only_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 4;
    instance.lifecycle = InstanceLifecycle::Waiting;
    instance.active_tokens = vec![TokenRecord {
        token_id: 3,
        node_index: 3,
        incoming_edge_index: Some(2),
        inclusive_join_hint: None,
    }];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[3].status = NodeRuntimeStatus::Executing;
    instance.waits.push(WaitRegistration {
        process_id: (Some("parallel_wait_only_frontier_plan".into())),
        node_index: 3,
        blocking_node_index: None,
        kind: WaitKind::ExternalEvent,
        event_kind: Some(BpmnEventKind::Message),
        event_reference: Some("parallel_wait_message".to_string()),
        event_name: Some("ParallelWaitMessage".to_string()),
        timer: None,
        condition_expression: None,
        deduplication_key: None,
    });

    let plan = plan_frontier_step(&package.processes[0], &instance);

    assert_eq!(plan.proposals.snapshot.first_runnable_token_index(), None);
    assert!(plan.proposals.execution_proposals.is_empty());
    assert_eq!(plan.action, BpmnFrontierPlanAction::WaitingExternalEvent);
}

#[test]
fn runtime_frontier_plan_returns_suspended_idle_outcome() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_host_and_wait_process(
            "parallel_suspended_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_suspended_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_suspended_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 5;
    instance.lifecycle = InstanceLifecycle::Suspended;
    instance.active_tokens = vec![TokenRecord {
        token_id: 4,
        node_index: 2,
        incoming_edge_index: Some(1),
        inclusive_join_hint: None,
    }];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = NodeRuntimeStatus::Cancelled;
    instance.suspend_reason = Some(SuspendReason::HostRequested);

    let plan = plan_frontier_step(&package.processes[0], &instance);

    assert_eq!(plan.proposals.snapshot.first_runnable_token_index(), None);
    assert!(plan.proposals.execution_proposals.is_empty());
    assert_eq!(
        plan.action,
        BpmnFrontierPlanAction::Suspended(Some(SuspendReason::HostRequested))
    );
}

#[test]
fn runtime_frontier_plan_returns_stalled_when_frontier_has_no_idle_outcome() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_dual_host_block_process(
            "parallel_stalled_frontier_plan",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_stalled_frontier_plan",
        BpmnInstanceInit::new("wf_parallel_stalled_frontier_plan", json!({}), 10),
    )
    .must("instance should be created");
    instance.sequence = 6;
    instance.lifecycle = InstanceLifecycle::Running;
    instance.active_tokens = vec![TokenRecord {
        token_id: 9,
        node_index: 2,
        incoming_edge_index: Some(1),
        inclusive_join_hint: None,
    }];
    instance.node_states[0].status = NodeRuntimeStatus::Completed;
    instance.node_states[1].status = NodeRuntimeStatus::Completed;
    instance.node_states[2].status = NodeRuntimeStatus::Cancelled;

    let plan = plan_frontier_step(&package.processes[0], &instance);

    assert_eq!(plan.proposals.snapshot.first_runnable_token_index(), None);
    assert!(plan.proposals.execution_proposals.is_empty());
    assert_eq!(plan.action, BpmnFrontierPlanAction::Stalled);
}
