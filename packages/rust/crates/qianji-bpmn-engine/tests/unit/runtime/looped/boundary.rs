use super::super::StubHost;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit,
    BpmnInstanceState, BpmnNodeKind, BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnStandardLoopSpec, BpmnTimerKind, BpmnTimerSpec, EventPollOutcome, InstanceLifecycle,
    NodeRuntimeStatus, PendingHostWorkResult, ProcessKey, ServiceTaskOutcome, advance_instance,
    apply_event_poll_outcome, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

fn standard_loop_non_interrupting_boundary_timer_process(process_id: &str) -> BpmnProcessSpec {
    BpmnProcessSpec::new(
        ProcessKey::new("pkg_runtime", process_id, format!("digest_{process_id}")),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::ServiceTask).with_repeat(
                BpmnRepeatSpec::StandardLoop(
                    BpmnStandardLoopSpec::new(true, Some(3)).with_loop_condition("not done"),
                ),
            ),
            BpmnNodeSpec::new(2, "review_timeout", BpmnNodeKind::BoundaryEvent)
                .with_boundary_attachment(1, false),
            BpmnNodeSpec::new(3, "approved_end", BpmnNodeKind::EndEvent),
            BpmnNodeSpec::new(4, "timeout_end", BpmnNodeKind::EndEvent),
        ],
        vec![
            BpmnEdgeSpec::new(0, 1, None::<&str>),
            BpmnEdgeSpec::new(1, 3, None::<&str>),
            BpmnEdgeSpec::new(2, 4, None::<&str>),
        ],
        vec![
            BpmnEventSpec::new(2, BpmnEventKind::Timer)
                .with_name("ReviewTimeout")
                .with_timer(BpmnTimerSpec::new(BpmnTimerKind::Duration, "PT30M")),
        ],
    )
}

fn assert_boundary_wait_armed(instance: &BpmnInstanceState) {
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
}

fn assert_requeued_owner_keeps_boundary_wait(instance: &BpmnInstanceState) {
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.standard_loops.len(), 1);
    assert_eq!(instance.standard_loops[0].completed_iterations, 1);
}

fn assert_boundary_branch_open(instance: &BpmnInstanceState) {
    assert_eq!(instance.pending_host_work.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens.len(), 2);
    assert!(
        instance
            .active_tokens
            .iter()
            .any(|token| token.node_index == 1)
    );
    assert!(
        instance
            .active_tokens
            .iter()
            .any(|token| token.node_index == 4)
    );
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Executing);
    assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.standard_loops[0].completed_iterations, 1);
    assert_eq!(instance.variables["amount"], json!(7));
    assert_eq!(instance.variables["timed_out"], json!(true));
}

fn assert_boundary_branch_drained(instance: &BpmnInstanceState) {
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.node_states[4].status, NodeRuntimeStatus::Completed);
}

fn assert_primary_path_resumed(instance: &BpmnInstanceState) {
    assert!(instance.standard_loops.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.node_states[3].status, NodeRuntimeStatus::Queued);
    assert_eq!(instance.variables["approved"], json!(true));
    assert_eq!(instance.variables["done"], json!(true));
    assert_eq!(instance.variables["timed_out"], json!(true));
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_standard_loop_non_interrupting_boundary_stays_armed_across_reentry() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![standard_loop_non_interrupting_boundary_timer_process(
            "loop_boundary_timer_non_interrupt",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "loop_boundary_timer_non_interrupt",
        BpmnInstanceInit::new(
            "wf_loop_boundary_timer_non_interrupt",
            json!({ "amount": 7, "done": false }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("first standard-loop iteration should block on host work");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_boundary_wait_armed(&instance);

    let token_id = instance.pending_host_work[0].token_id;
    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome { data: json!({}) }),
        100,
    )
    .must("first standard-loop completion should re-queue the owner");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_requeued_owner_keeps_boundary_wait(&instance);

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("re-queued standard-loop owner should block again without losing the boundary");
    assert_eq!(
        blocked_again,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_boundary_wait_armed(&instance);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Executing);
    assert_eq!(instance.standard_loops[0].completed_iterations, 1);

    let boundary_resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        120,
    )
    .must("boundary timer should open a concurrent branch without cancelling the owner");
    assert_eq!(boundary_resumed, BpmnAdvanceOutcome::Advanced);
    assert_boundary_branch_open(&instance);

    let blocked_third = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("boundary branch should drain while the loop owner remains blocked");
    assert_eq!(
        blocked_third,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_boundary_branch_drained(&instance);

    let token_id = instance.pending_host_work[0].token_id;
    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "done": true, "approved": true }),
        }),
        140,
    )
    .must("final standard-loop completion should clear the boundary wait and route onward");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_primary_path_resumed(&instance);

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("standard-loop primary path should complete after the boundary branch drains");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}
