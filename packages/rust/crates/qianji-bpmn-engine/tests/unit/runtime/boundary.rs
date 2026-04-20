use super::{StubHost, boundary_timer_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnPackage, BpmnTimerKind,
    EventPollOutcome, InstanceLifecycle, PendingHostWorkKind, PendingHostWorkResult,
    UserTaskOutcome, WaitKind, advance_instance, apply_event_poll_outcome,
    apply_pending_host_work_result, build_event_poll_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_interrupting_boundary_timer_arms_wait_and_routes_timeout_path() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![boundary_timer_process("boundary_timer")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_timer",
        BpmnInstanceInit::new("wf_boundary_timer", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the boundary timer");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].node_index, 1);
    assert_eq!(pending[0].kind, PendingHostWorkKind::User);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.waits[0].kind, WaitKind::Timer);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Timer));
    let timer = instance.waits[0]
        .timer
        .as_ref()
        .must("boundary wait should preserve timer snapshot");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");

    let poll_request = build_event_poll_request(&instance)
        .must("boundary timer wait should materialize an event poll request");
    assert_eq!(poll_request.gateway_node_index, None);
    assert_eq!(poll_request.waits, instance.waits);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        100,
    )
    .must("timer outcome should interrupt the blocked task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "timed_out": true })
    );

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timeout path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_host_completion_clears_interrupting_boundary_timer_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![boundary_timer_process("boundary_timer")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_timer",
        BpmnInstanceInit::new("wf_boundary_timer", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the boundary timer");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.waits.len(), 1);
    let token_id = instance.pending_host_work[0].token_id;

    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("host completion should win over the boundary timer");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}
