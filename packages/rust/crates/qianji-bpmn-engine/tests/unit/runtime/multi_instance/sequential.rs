use super::super::{
    StubHost, dmn_fixture_definition, sequential_multi_instance_business_rule_process,
    sequential_multi_instance_data_binding_business_rule_process,
    sequential_multi_instance_data_binding_process, sequential_multi_instance_process,
    sequential_multi_instance_process_with_completion_condition,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnProcessSpec, BpmnRepeatSpec, BpmnSequentialMultiInstanceSpec,
    BpmnTimerKind, BpmnTimerSpec, PendingHostWork, PendingHostWorkKind, PendingHostWorkResult,
    ProcessKey, ServiceTaskOutcome, advance_instance, apply_event_poll_outcome,
    apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_repeats_until_cardinality_is_reached() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_process(
            "multi_instance_service",
            BpmnNodeKind::ServiceTask,
            3,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_service",
        BpmnInstanceInit::new("wf_multi_instance", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(201);

    for completed in 0..3 {
        let blocked = advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("sequential multi-instance iteration should block on host work");
        let pending = instance
            .pending_host_work
            .first()
            .cloned()
            .must("multi-instance iteration should register pending host work");
        assert_eq!(
            blocked,
            BpmnAdvanceOutcome::BlockedOnHost(vec![pending.clone()])
        );
        assert_eq!(
            pending,
            PendingHostWork {
                token_id: instance.active_tokens[0].token_id,
                node_index: 1,
                kind: PendingHostWorkKind::Service,
                decision: None,
                work_id: None,
            }
        );
        assert_eq!(instance.sequential_multi_instances.len(), 1);
        assert_eq!(instance.sequential_multi_instances[0].total_iterations, 3);
        assert_eq!(
            instance.sequential_multi_instances[0].completed_iterations,
            completed
        );

        let resumed = apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending.token_id,
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({ "last_completed": completed }),
            }),
            250 + u64::from(completed),
        )
        .must("host completion should advance sequential multi-instance");
        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);

        if completed < 2 {
            assert_eq!(instance.active_tokens[0].node_index, 1);
            assert_eq!(
                instance.node_states[1].status,
                qianji_bpmn_engine::NodeRuntimeStatus::Queued
            );
            assert_eq!(
                instance.sequential_multi_instances[0].completed_iterations,
                completed + 1
            );
        } else {
            assert!(instance.sequential_multi_instances.is_empty());
            assert_eq!(instance.active_tokens[0].node_index, 2);
            assert_eq!(
                instance.node_states[1].status,
                qianji_bpmn_engine::NodeRuntimeStatus::Completed
            );
            assert_eq!(
                instance.node_states[2].status,
                qianji_bpmn_engine::NodeRuntimeStatus::Queued
            );
        }
    }

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("multi-instance should complete after the final routed end event");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_zero_cardinality_skips_host_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_process(
            "multi_instance_zero",
            BpmnNodeKind::ServiceTask,
            0,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_zero",
        BpmnInstanceInit::new("wf_multi_instance_zero", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(220))
        .await
        .must("zero-cardinality sequential multi-instance should skip host work");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.sequential_multi_instances.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_completion_condition_stops_future_iterations() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_process_with_completion_condition(
            "multi_instance_completion_condition",
            BpmnNodeKind::ServiceTask,
            5,
            "completed >= 2",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_completion_condition",
        BpmnInstanceInit::new(
            "wf_multi_instance_completion_condition",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(224);

    for completed in 0..2 {
        let blocked = advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("sequential completion-condition iteration should block on host work");
        assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
        let pending = instance
            .pending_host_work
            .first()
            .cloned()
            .must("multi-instance completion-condition iteration should register work");

        let resumed = apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending.token_id,
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({ "last_completed": completed }),
            }),
            280 + u64::try_from(completed).must("completed index fits in u64"),
        )
        .must("host completion should advance sequential completion-condition state");
        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    }

    assert!(instance.pending_host_work.is_empty());
    assert!(instance.sequential_multi_instances.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "last_completed": 1 })
    );

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("completion-condition sequential path should finish");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}

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
async fn runtime_sequential_multi_instance_business_rule_executes_locally() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![sequential_multi_instance_business_rule_process(
                "multi_instance_local_business_rule",
                3,
            )],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_local_business_rule",
        BpmnInstanceInit::new(
            "wf_multi_instance_local_business_rule",
            json!({ "tier": "gold" }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(280))
        .await
        .must("local business-rule multi-instance should complete without host work");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.sequential_multi_instances.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(instance.sequence, 6);
    assert_eq!(instance.updated_at_ms, 280);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_data_binding_aggregates_array_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![sequential_multi_instance_data_binding_process(
            "multi_instance_data_binding",
            BpmnNodeKind::ServiceTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_multi_instance_data_binding",
            json!({ "items": [2, 4, 6] }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(281);

    for expected_result in [20, 40, 60] {
        let blocked = advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("sequential data-binding iteration should block on host work");
        assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
        let pending = instance
            .pending_host_work
            .first()
            .cloned()
            .must("data-binding iteration should register pending host work");

        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending.token_id,
            PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({ "result": expected_result }),
            }),
            300 + u64::try_from(expected_result).must("expected result fits in u64"),
        )
        .must("host completion should capture data-binding output");
    }

    assert_eq!(
        instance.variables,
        json!({
            "items": [2, 4, 6],
            "results": [20, 40, 60],
        })
    );
    assert!(instance.variables.get("item").is_none());
    assert!(instance.variables.get("result").is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_sequential_multi_instance_data_binding_business_rule_uses_iteration_overlay() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![
                sequential_multi_instance_data_binding_business_rule_process(
                    "multi_instance_data_binding_business_rule",
                ),
            ],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "multi_instance_data_binding_business_rule",
        BpmnInstanceInit::new(
            "wf_multi_instance_data_binding_business_rule",
            json!({ "tiers": ["gold", "silver"] }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(282))
        .await
        .must("data-bound business-rule multi-instance should execute locally");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(
        instance.variables,
        json!({
            "tiers": ["gold", "silver"],
            "decisions": ["approve", "review"],
        })
    );
    assert!(instance.variables.get("tier").is_none());
    assert!(instance.variables.get("approval").is_none());
}
