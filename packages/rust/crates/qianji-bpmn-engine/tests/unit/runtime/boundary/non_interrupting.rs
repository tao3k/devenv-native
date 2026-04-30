use super::super::{
    StubHost, non_interrupting_boundary_conditional_process,
    non_interrupting_boundary_external_process, non_interrupting_boundary_timer_process,
};
use super::helpers::{
    assert_non_interrupting_boundary_branch_drained, assert_non_interrupting_boundary_branch_open,
    assert_non_interrupting_boundary_external_wait, assert_non_interrupting_primary_path_resumed,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnPackage, EventPollOutcome,
    InstanceLifecycle, NodeRuntimeStatus, PendingHostWorkResult, UserTaskOutcome, WaitKind,
    advance_instance, apply_event_poll_outcome, apply_pending_host_work_result,
    build_event_poll_request, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_non_interrupting_boundary_timer_spawns_concurrent_timeout_path() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![non_interrupting_boundary_timer_process(
            "boundary_timer_non_interrupt",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_timer_non_interrupt",
        BpmnInstanceInit::new(
            "wf_boundary_timer_non_interrupt",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the non-interrupting boundary timer");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));

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
    .must("timer outcome should open the timeout path without cancelling the task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_non_interrupting_boundary_branch_open(&instance, "timed_out");

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timeout branch should drain while original task remains blocked");

    assert_eq!(
        blocked_again,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_non_interrupting_boundary_branch_drained(&instance);

    let token_id = instance.pending_host_work[0].token_id;
    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        120,
    )
    .must("host completion should still route the original task path");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_non_interrupting_primary_path_resumed(&instance, "timed_out");

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("primary path should still complete after timeout branch ran");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_host_completion_clears_non_interrupting_boundary_timer_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![non_interrupting_boundary_timer_process(
            "boundary_timer_non_interrupt",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_timer_non_interrupt",
        BpmnInstanceInit::new(
            "wf_boundary_timer_non_interrupt",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the non-interrupting boundary timer");
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
    .must("host completion should clear the unresolved non-interrupting boundary wait");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Idle);
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_non_interrupting_boundary_message_spawns_concurrent_boundary_path() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![non_interrupting_boundary_external_process(
            "boundary_message_non_interrupt",
            BpmnEventKind::Message,
            "review_message",
            "ReviewMessage",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_message_non_interrupt",
        BpmnInstanceInit::new(
            "wf_boundary_message_non_interrupt",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the non-interrupting boundary message");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_non_interrupting_boundary_external_wait(
        &instance,
        BpmnEventKind::Message,
        "review_message",
        "ReviewMessage",
    );

    let poll_request = build_event_poll_request(&instance)
        .must("non-interrupting boundary message should materialize an event poll request");
    assert_eq!(poll_request.gateway_node_index, None);
    assert_eq!(poll_request.waits, instance.waits);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "escalated": true }),
        },
        100,
    )
    .must("message outcome should open the boundary path without cancelling the task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_non_interrupting_boundary_branch_open(&instance, "escalated");

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("boundary path should drain while original task remains blocked");
    assert_eq!(
        blocked_again,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_non_interrupting_boundary_branch_drained(&instance);

    let token_id = instance.pending_host_work[0].token_id;
    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        120,
    )
    .must("host completion should still route the original task path");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_non_interrupting_primary_path_resumed(&instance, "escalated");

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("primary path should still complete after boundary branch ran");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_non_interrupting_conditional_boundary_spawns_when_condition_becomes_true() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![non_interrupting_boundary_conditional_process(
            "boundary_conditional_non_interrupt",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_conditional_non_interrupt",
        BpmnInstanceInit::new(
            "wf_boundary_conditional_non_interrupt",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the non-interrupting boundary conditional wait");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.waits[0].kind, WaitKind::Conditional);
    assert_eq!(
        instance.waits[0].event_kind,
        Some(BpmnEventKind::Conditional)
    );
    assert_eq!(
        instance.waits[0].condition_expression.as_deref(),
        Some("escalated")
    );

    let poll_request = build_event_poll_request(&instance)
        .must("non-interrupting boundary conditional should materialize an event poll request");
    assert_eq!(poll_request.gateway_node_index, None);
    assert_eq!(poll_request.waits, instance.waits);

    let still_waiting = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "escalated": false }),
        },
        100,
    )
    .must("false conditional data should keep the task blocked");
    assert_eq!(still_waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "escalated": true }),
        },
        120,
    )
    .must("true conditional data should open the boundary path without cancelling the task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_non_interrupting_boundary_branch_open(&instance, "escalated");
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_host_completion_clears_non_interrupting_boundary_signal_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![non_interrupting_boundary_external_process(
            "boundary_signal_non_interrupt",
            BpmnEventKind::Signal,
            "review_signal",
            "ReviewSignal",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "boundary_signal_non_interrupt",
        BpmnInstanceInit::new(
            "wf_boundary_signal_non_interrupt",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the non-interrupting boundary signal");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_non_interrupting_boundary_external_wait(
        &instance,
        BpmnEventKind::Signal,
        "review_signal",
        "ReviewSignal",
    );
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
    .must("host completion should clear the unresolved non-interrupting boundary signal");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Idle);
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}
