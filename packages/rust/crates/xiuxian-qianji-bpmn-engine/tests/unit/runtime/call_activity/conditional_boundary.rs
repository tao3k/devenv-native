use super::{
    EMBEDDED_REVIEW_PROCESS_ID, StubHost, TRANSACTION_PROCESS_ID, node_index,
    parsed_fixture_package,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, EventPollOutcome, InstanceLifecycle,
    NodeRuntimeStatus, WaitKind, advance_instance, apply_event_poll_outcome,
    build_event_poll_request, create_instance,
};

#[derive(Clone, Copy)]
struct ConditionalBoundaryCase {
    fixture: &'static str,
    instance_id: &'static str,
    child_process_id: &'static str,
    owner_id: &'static str,
    boundary_id: &'static str,
    boundary_end_id: &'static str,
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_embedded_subprocess_conditional_boundary_routes_parent_path() {
    assert_subprocess_conditional_boundary_routes(ConditionalBoundaryCase {
        fixture: "embedded-subprocess-conditional-boundary.bpmn",
        instance_id: "wf_embedded_subprocess_conditional_boundary",
        child_process_id: EMBEDDED_REVIEW_PROCESS_ID,
        owner_id: "inline_review",
        boundary_id: "review_condition",
        boundary_end_id: "condition_end",
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_conditional_boundary_routes_parent_path() {
    assert_subprocess_conditional_boundary_routes(ConditionalBoundaryCase {
        fixture: "call-activity-conditional-boundary.bpmn",
        instance_id: "wf_call_activity_conditional_boundary",
        child_process_id: "child_process",
        owner_id: "invoke_review",
        boundary_id: "review_condition",
        boundary_end_id: "condition_end",
    })
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_conditional_boundary_routes_parent_path() {
    assert_subprocess_conditional_boundary_routes(ConditionalBoundaryCase {
        fixture: "transaction-conditional-boundary.bpmn",
        instance_id: "wf_transaction_conditional_boundary",
        child_process_id: TRANSACTION_PROCESS_ID,
        owner_id: "payment_tx",
        boundary_id: "tx_condition",
        boundary_end_id: "condition_end",
    })
    .await;
}

async fn assert_subprocess_conditional_boundary_routes(case: ConditionalBoundaryCase) {
    let package = Arc::new(parsed_fixture_package(case.fixture));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(case.instance_id, json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("subprocess-like owner should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_conditional_boundary_wait_armed(package.as_ref(), &instance, &case);

    let poll_request = build_event_poll_request(&instance)
        .must("parent conditional boundary wait should materialize an event poll request");
    assert_eq!(poll_request.gateway_node_index, None);
    assert_eq!(poll_request.waits.len(), 1);
    assert_eq!(poll_request.waits, instance.call_stack[0].waits);

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
    .must("false conditional data should keep the child process blocked");
    assert_eq!(still_waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_parent_conditional_boundary_wait_armed(package.as_ref(), &instance, &case);

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
    .must("true conditional data should cancel the child scope and route the parent boundary");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_parent_conditional_boundary_route_open(package.as_ref(), &instance, &case);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("conditional boundary path should complete");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

fn assert_parent_conditional_boundary_wait_armed(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &xiuxian_qianji_bpmn_engine::BpmnInstanceState,
    case: &ConditionalBoundaryCase,
) {
    assert_eq!(instance.process.process_id.as_ref(), case.child_process_id);
    assert_eq!(instance.call_stack.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.call_stack[0].waits.len(), 1);
    let wait = &instance.call_stack[0].waits[0];
    assert_eq!(wait.process_id.as_deref(), Some("main_process"));
    assert_eq!(
        wait.node_index,
        node_index(package, "main_process", case.boundary_id)
    );
    assert_eq!(
        wait.blocking_node_index,
        Some(node_index(package, "main_process", case.owner_id))
    );
    assert_eq!(wait.kind, WaitKind::Conditional);
    assert_eq!(wait.event_kind, Some(BpmnEventKind::Conditional));
    assert_eq!(wait.condition_expression.as_deref(), Some("escalated"));
    assert!(wait.timer.is_none());
}

fn assert_parent_conditional_boundary_route_open(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &xiuxian_qianji_bpmn_engine::BpmnInstanceState,
    case: &ConditionalBoundaryCase,
) {
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(package, "main_process", case.boundary_end_id)
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", case.owner_id) as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", case.boundary_id) as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", case.boundary_end_id) as usize]
            .status,
        NodeRuntimeStatus::Queued
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "success_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "escalated": true })
    );
}
