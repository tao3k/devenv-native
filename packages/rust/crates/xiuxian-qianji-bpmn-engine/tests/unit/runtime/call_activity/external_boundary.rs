use super::{StubHost, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnTimerKind, EventPollOutcome,
    InstanceLifecycle, NodeRuntimeStatus, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    apply_event_poll_outcome, build_event_poll_request, create_instance,
};

fn assert_parent_timer_boundary_wait_armed(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &xiuxian_qianji_bpmn_engine::BpmnInstanceState,
) {
    assert_eq!(instance.process.process_id.as_ref(), "child_process");
    assert_eq!(instance.call_stack.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.call_stack[0].waits.len(), 1);
    assert_eq!(
        instance.call_stack[0].waits[0].process_id.as_deref(),
        Some("main_process")
    );
    assert_eq!(
        instance.call_stack[0].waits[0].node_index,
        node_index(package, "main_process", "review_timeout")
    );
    assert_eq!(
        instance.call_stack[0].waits[0].blocking_node_index,
        Some(node_index(package, "main_process", "invoke_review"))
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

fn assert_parent_boundary_poll_request(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &xiuxian_qianji_bpmn_engine::BpmnInstanceState,
) {
    let poll_request = build_event_poll_request(instance)
        .must("parent boundary wait should materialize an event poll request");
    assert_eq!(poll_request.gateway_node_index, None);
    assert_eq!(poll_request.waits.len(), 1);
    assert_eq!(
        poll_request.waits[0].process_id.as_deref(),
        Some("main_process")
    );
    assert_eq!(
        poll_request.waits[0].node_index,
        node_index(package, "main_process", "review_timeout")
    );
}

fn assert_parent_boundary_route_open(
    package: &xiuxian_qianji_bpmn_engine::BpmnPackage,
    instance: &xiuxian_qianji_bpmn_engine::BpmnInstanceState,
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
        instance.node_states[node_index(package, "main_process", "invoke_review") as usize].status,
        NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "review_timeout") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "timeout_end") as usize].status,
        NodeRuntimeStatus::Queued
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "success_end") as usize].status,
        NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "timed_out": true })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_interrupting_timer_boundary_routes_parent_path() {
    let package = Arc::new(parsed_fixture_package("call-activity-timer-boundary.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_call_activity_timer_boundary",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);
    assert_parent_boundary_poll_request(package.as_ref(), &instance);

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
    .must("timer outcome should cancel the child process and route the parent boundary");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_parent_boundary_route_open(package.as_ref(), &instance);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timeout boundary path should complete");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_success_clears_interrupting_timer_boundary_wait() {
    let package = Arc::new(parsed_fixture_package("call-activity-timer-boundary.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_call_activity_timer_boundary_success",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);
    let token_id = instance.pending_host_work[0].token_id;

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("host completion should resume the child process");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_parent_timer_boundary_wait_armed(package.as_ref(), &instance);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should complete normally and clear the parent timer boundary wait");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "invoke_review") as usize].status,
        NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "review_timeout") as usize]
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
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}
