use super::super::super::StubHost;
use super::helpers::sequential_multi_instance_non_interrupting_boundary_process;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec, BpmnSequentialMultiInstanceSpec,
    BpmnTimerKind, BpmnTimerSpec, ProcessKey, advance_instance, apply_event_poll_outcome,
    apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_interrupting_boundary_timer_clears_sequential_multi_instance_state() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "multi_instance_boundary",
            "digest_multi_instance_boundary",
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::UserTask).with_repeat(
                BpmnRepeatSpec::SequentialMultiInstance(BpmnSequentialMultiInstanceSpec::new(3)),
            ),
            BpmnNodeSpec::new(2, "review_timeout", BpmnNodeKind::BoundaryEvent)
                .with_boundary_attachment(1, true),
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
    );
    let package = Arc::new(BpmnPackage::new("pkg_runtime", vec![process]));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_boundary",
        BpmnInstanceInit::new("wf_multi_instance_boundary", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(230);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("user task should block and arm the boundary timer");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.sequential_multi_instances.len(), 1);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        260,
    )
    .must("timer outcome should interrupt the blocked sequential multi-instance task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert!(instance.sequential_multi_instances.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 4);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timeout path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_handoff_keeps_non_interrupting_boundary_wait_armed() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_non_interrupting_boundary_process(
            "sequential_multi_instance_non_interrupt_boundary",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "sequential_multi_instance_non_interrupt_boundary",
        BpmnInstanceInit::new(
            "wf_sequential_multi_instance_non_interrupt_boundary",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(231);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("first sequential iteration should block and arm the boundary timer");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));

    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        qianji_bpmn_engine::PendingHostWorkResult::User(qianji_bpmn_engine::UserTaskOutcome {
            data: json!({ "approved": true, "completed_iteration": 0 }),
        }),
        300,
    )
    .must("non-final sequential completion should preserve the owner-level non-interrupting boundary wait");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.pending_host_work.len(), 0);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.sequential_multi_instances.len(), 1);
    assert_eq!(
        instance.sequential_multi_instances[0].completed_iterations,
        1
    );

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("queued sequential owner should still start the next iteration while the boundary wait stays armed");
    let next_pending = match blocked_again {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(next_pending.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_non_interrupting_boundary_timer_opens_sequential_multi_instance_branch() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_non_interrupting_boundary_process(
            "sequential_multi_instance_non_interrupt_boundary_open",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "sequential_multi_instance_non_interrupt_boundary_open",
        BpmnInstanceInit::new(
            "wf_sequential_multi_instance_non_interrupt_boundary_open",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(232);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("sequential owner should block and arm the non-interrupting boundary timer");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.waits.len(), 1);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        320,
    )
    .must("timer outcome should open a concurrent boundary path without cancelling the sequential owner");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.pending_host_work.len(), 1);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.sequential_multi_instances.len(), 1);
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
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.variables["timed_out"], json!(true));

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("boundary branch should drain while the sequential owner remains blocked");
    assert_eq!(
        blocked_again,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}
