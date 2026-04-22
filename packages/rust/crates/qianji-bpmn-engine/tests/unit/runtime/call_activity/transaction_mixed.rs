use super::{StubHost, TRANSACTION_PROCESS_ID, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnTimerKind, EventPollOutcome,
    InstanceLifecycle, NodeRuntimeStatus, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    apply_event_poll_outcome, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

fn assert_parent_timer_boundary_wait_armed(
    package: &qianji_bpmn_engine::BpmnPackage,
    instance: &qianji_bpmn_engine::BpmnInstanceState,
) {
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(instance.call_stack.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.call_stack[0].waits.len(), 1);
    assert_eq!(
        instance.call_stack[0].waits[0].process_id.as_deref(),
        Some("main_process")
    );
    assert_eq!(
        instance.call_stack[0].waits[0].node_index,
        node_index(package, "main_process", "tx_timeout")
    );
    assert_eq!(
        instance.call_stack[0].waits[0].blocking_node_index,
        Some(node_index(package, "main_process", "payment_tx"))
    );
    assert_eq!(
        instance.call_stack[0].waits[0].event_kind,
        Some(BpmnEventKind::Timer)
    );
    let timer = instance.call_stack[0].waits[0]
        .timer
        .as_ref()
        .must("parent boundary wait should preserve timer metadata");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");
}

fn assert_timeout_route_open(
    package: &qianji_bpmn_engine::BpmnPackage,
    instance: &qianji_bpmn_engine::BpmnInstanceState,
) {
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(package, "main_process", "timeout_end")
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "payment_tx") as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "tx_timeout") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "tx_error_specific") as usize]
            .status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "tx_error_catch_all") as usize]
            .status,
        NodeRuntimeStatus::Cancelled
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_mixed_timer_boundary_routes_and_cancels_error_siblings() {
    let package = Arc::new(parsed_fixture_package("transaction-mixed-boundaries.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_mixed_timeout", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);

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
    .must("timer outcome should route the timeout path");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_timeout_route_open(package.as_ref(), &instance);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timeout path should complete");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_mixed_error_route_clears_timer_wait() {
    let package = Arc::new(parsed_fixture_package("transaction-mixed-boundaries.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_mixed_error", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);
    let token_id = instance.pending_host_work[0].token_id;

    assert_eq!(
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": false, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the transaction child"),
        BpmnAdvanceOutcome::Advanced
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction error end should route through matching error boundaries");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_timeout") as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_specific") as usize]
            .status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": false, "reviewer": "alice" })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_mixed_success_clears_all_boundaries() {
    let package = Arc::new(parsed_fixture_package("transaction-mixed-boundaries.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_mixed_success", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);
    let token_id = instance.pending_host_work[0].token_id;

    assert_eq!(
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the transaction child"),
        BpmnAdvanceOutcome::Advanced
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should complete normally and cancel sibling boundaries");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_timeout") as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_specific") as usize]
            .status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "timeout_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );
}
