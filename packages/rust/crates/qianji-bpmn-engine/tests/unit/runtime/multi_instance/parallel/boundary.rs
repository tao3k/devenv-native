use super::helpers::parallel_multi_instance_non_interrupting_boundary_process;
use crate::runtime::{StubHost, runtime_optional_output_io};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnParallelMultiInstanceSpec, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnTimerKind, BpmnTimerSpec, ProcessKey, advance_instance, apply_event_poll_outcome,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

fn assert_parallel_multi_instance_non_interrupting_boundary_branch_open(
    instance: &qianji_bpmn_engine::BpmnInstanceState,
) {
    assert_eq!(instance.pending_host_work.len(), 3);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.parallel_multi_instances.len(), 1);
    assert_eq!(instance.active_tokens.len(), 4);
    assert_eq!(
        instance
            .active_tokens
            .iter()
            .filter(|token| token.node_index == 1)
            .count(),
        3
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
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "timed_out": true })
    );
}

fn assert_parallel_multi_instance_non_interrupting_boundary_branch_drained(
    instance: &qianji_bpmn_engine::BpmnInstanceState,
) {
    assert_eq!(instance.pending_host_work.len(), 3);
    assert_eq!(instance.active_tokens.len(), 3);
    assert!(
        instance
            .active_tokens
            .iter()
            .all(|token| token.node_index == 1)
    );
    assert_eq!(
        instance.node_states[4].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_interrupting_boundary_timer_clears_parallel_multi_instance_state() {
    let process = BpmnProcessSpec::new(
        ProcessKey::new(
            "pkg_runtime",
            "parallel_multi_instance_boundary",
            "digest_parallel_multi_instance_boundary",
        ),
        vec![
            BpmnNodeSpec::new(0, "start", BpmnNodeKind::StartEvent),
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::UserTask)
                .with_repeat(BpmnRepeatSpec::ParallelMultiInstance(
                    BpmnParallelMultiInstanceSpec::new(3),
                ))
                .with_task_io(runtime_optional_output_io()),
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
        "parallel_multi_instance_boundary",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_boundary",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(223);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel multi-instance user task should block and arm the boundary timer");
    match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => assert_eq!(pending.len(), 3),
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    }
    let expected_winner_token_id = instance
        .active_tokens
        .iter()
        .map(|token| token.token_id)
        .min()
        .must("parallel multi-instance boundary wait should keep active tokens");
    assert_eq!(instance.parallel_multi_instances.len(), 1);
    assert_eq!(instance.waits.len(), 1);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        270,
    )
    .must("timer outcome should interrupt the blocked parallel multi-instance task");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert!(instance.parallel_multi_instances.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].token_id, expected_winner_token_id);
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
async fn runtime_non_interrupting_boundary_timer_opens_parallel_multi_instance_branch() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_non_interrupting_boundary_process(
            "parallel_multi_instance_non_interrupt_boundary",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_non_interrupt_boundary",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_non_interrupt_boundary",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(224);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel multi-instance user task should block and arm the non-interrupting boundary timer");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 3);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 2);
    assert_eq!(instance.waits[0].blocking_node_index, Some(1));

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "timed_out": true }),
        },
        270,
    )
    .must("timer outcome should open a concurrent boundary path without cancelling the parallel multi-instance owner");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_parallel_multi_instance_non_interrupting_boundary_branch_open(&instance);

    let blocked_again = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "boundary branch should drain while the parallel multi-instance owner remains blocked",
        );
    assert_eq!(
        blocked_again,
        BpmnAdvanceOutcome::BlockedOnHost(instance.pending_host_work.clone())
    );
    assert_parallel_multi_instance_non_interrupting_boundary_branch_drained(&instance);

    for (completed, pending_work) in pending.iter().enumerate() {
        let resumed = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            qianji_bpmn_engine::PendingHostWorkResult::User(qianji_bpmn_engine::UserTaskOutcome {
                data: json!({ "approved": true, "completed_iteration": completed }),
            }),
            320 + u64::try_from(completed).must("completed index fits in u64"),
        )
        .must("parallel multi-instance completion should still route the primary path");
        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    }

    assert!(instance.parallel_multi_instances.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.variables["timed_out"], json!(true));
    assert_eq!(instance.variables["approved"], json!(true));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel multi-instance primary path should still complete after the boundary branch drains");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_host_completion_keeps_non_interrupting_boundary_wait_until_last_iteration()
 {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_non_interrupting_boundary_process(
            "parallel_multi_instance_non_interrupt_boundary",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_non_interrupt_boundary",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_non_interrupt_boundary_completion",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(225);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel multi-instance user task should block and arm the non-interrupting boundary timer");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 3);
    assert_eq!(instance.waits.len(), 1);

    for (completed, pending_work) in pending.iter().take(2).enumerate() {
        let resumed = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            qianji_bpmn_engine::PendingHostWorkResult::User(qianji_bpmn_engine::UserTaskOutcome {
                data: json!({ "approved": true, "completed_iteration": completed }),
            }),
            360 + u64::try_from(completed).must("completed index fits in u64"),
        )
        .must("non-final parallel multi-instance completion should keep the owner-level boundary wait armed");

        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
        assert_eq!(instance.waits.len(), 1);
        assert_eq!(instance.waits[0].node_index, 2);
        assert_eq!(instance.waits[0].blocking_node_index, Some(1));
        assert_eq!(instance.parallel_multi_instances.len(), 1);
        assert_eq!(instance.pending_host_work.len(), 2 - completed);
        assert_eq!(instance.active_tokens.len(), 2 - completed);
        assert!(
            instance
                .active_tokens
                .iter()
                .all(|token| token.node_index == 1)
        );
    }

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[2].token_id,
        qianji_bpmn_engine::PendingHostWorkResult::User(qianji_bpmn_engine::UserTaskOutcome {
            data: json!({ "approved": true, "completed_iteration": 2 }),
        }),
        362,
    )
    .must("final parallel multi-instance completion should clear the unresolved non-interrupting boundary wait");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.parallel_multi_instances.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 3);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[2].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[3].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );
    assert_eq!(instance.variables["approved"], json!(true));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("final routed end event should complete the instance");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}
