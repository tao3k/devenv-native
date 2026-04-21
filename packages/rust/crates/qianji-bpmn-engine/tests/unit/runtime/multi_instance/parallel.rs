use super::super::{
    StubHost, dmn_fixture_definition, parallel_multi_instance_business_rule_process,
    parallel_multi_instance_data_binding_process, parallel_multi_instance_process,
    parallel_multi_instance_process_with_completion_condition,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEdgeSpec, BpmnEventKind, BpmnEventSpec, BpmnInstanceInit, BpmnNodeKind,
    BpmnNodeSpec, BpmnPackage, BpmnParallelMultiInstanceSpec, BpmnProcessSpec, BpmnRepeatSpec,
    BpmnTimerKind, BpmnTimerSpec, ProcessKey, ServiceTaskOutcome, advance_instance,
    apply_event_poll_outcome, apply_pending_host_work_result, build_pending_host_work_requests,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_blocks_all_iterations_before_final_routing() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_process(
            "parallel_multi_instance_service",
            BpmnNodeKind::ServiceTask,
            3,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_service",
        BpmnInstanceInit::new("wf_parallel_multi_instance", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(221);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel multi-instance should materialize every iteration before blocking");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };

    assert_eq!(pending.len(), 3);
    assert_eq!(instance.parallel_multi_instances.len(), 1);
    assert_eq!(instance.parallel_multi_instances[0].total_iterations, 3);
    assert_eq!(instance.parallel_multi_instances[0].completed_iterations, 0);
    assert_eq!(
        instance.parallel_multi_instances[0].active_iterations.len(),
        3
    );
    assert_eq!(instance.active_tokens.len(), 3);
    assert!(
        instance
            .active_tokens
            .iter()
            .all(|token| token.node_index == 1)
    );

    for (completed, pending_work) in pending.iter().enumerate() {
        let resumed = apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({ "last_completed": completed }),
            }),
            260 + u64::try_from(completed).must("completed index fits in u64"),
        )
        .must("parallel multi-instance completion should advance");
        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);

        if completed < 2 {
            assert_eq!(instance.parallel_multi_instances.len(), 1);
            assert_eq!(
                instance.parallel_multi_instances[0].completed_iterations,
                u32::try_from(completed + 1).must("completed iterations fit in u32")
            );
            assert_eq!(instance.pending_host_work.len(), 2 - completed);
            assert_eq!(instance.active_tokens.len(), 2 - completed);
            assert!(
                instance
                    .active_tokens
                    .iter()
                    .all(|token| token.node_index == 1)
            );
        } else {
            assert!(instance.parallel_multi_instances.is_empty());
            assert!(instance.pending_host_work.is_empty());
            assert_eq!(instance.active_tokens.len(), 1);
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
        .must("parallel multi-instance should complete after the final routed end event");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_zero_cardinality_skips_host_work() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_process(
            "parallel_multi_instance_zero",
            BpmnNodeKind::ServiceTask,
            0,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_zero",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_zero",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(222))
        .await
        .must("zero-cardinality parallel multi-instance should skip host work");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.parallel_multi_instances.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_completion_condition_cancels_siblings() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_process_with_completion_condition(
            "parallel_multi_instance_completion_condition",
            BpmnNodeKind::ServiceTask,
            3,
            "completed >= 1",
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_completion_condition",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_completion_condition",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(222);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel completion-condition path should block on host work");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 3);

    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[1].token_id,
        qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "winner": true }),
        }),
        261,
    )
    .must("one completion should satisfy the bounded completion condition");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.parallel_multi_instances.is_empty());
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].token_id, pending[1].token_id);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "winner": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("completion-condition parallel path should finish");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
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
            BpmnNodeSpec::new(1, "review", BpmnNodeKind::UserTask).with_repeat(
                BpmnRepeatSpec::ParallelMultiInstance(BpmnParallelMultiInstanceSpec::new(3)),
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
async fn runtime_parallel_multi_instance_business_rule_executes_locally() {
    let package = Arc::new(
        BpmnPackage::new(
            "pkg_runtime",
            vec![parallel_multi_instance_business_rule_process(
                "parallel_multi_instance_local_business_rule",
                3,
            )],
        )
        .with_dmn_decisions(vec![dmn_fixture_definition(
            "simple-unique-eligibility.dmn",
        )]),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_local_business_rule",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_local_business_rule",
            json!({ "tier": "gold" }),
            10,
        ),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(240))
        .await
        .must("parallel business rule multi-instance should execute locally");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.parallel_multi_instances.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "tier": "gold", "approval": "approve" })
    );
    assert_eq!(
        instance.lifecycle,
        qianji_bpmn_engine::InstanceLifecycle::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_parallel_multi_instance_data_binding_aggregates_object_output() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![parallel_multi_instance_data_binding_process(
            "parallel_multi_instance_data_binding",
            BpmnNodeKind::ServiceTask,
        )],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "parallel_multi_instance_data_binding",
        BpmnInstanceInit::new(
            "wf_parallel_multi_instance_data_binding",
            json!({
                "assignments": {
                    "alpha": "approve",
                    "beta": "review",
                }
            }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(241);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parallel data-binding path should block on host work");
    let pending = match blocked {
        BpmnAdvanceOutcome::BlockedOnHost(pending) => pending,
        other => panic!("expected blocked-on-host outcome, got {other:?}"),
    };
    assert_eq!(pending.len(), 2);

    for pending_work in pending {
        let item = build_pending_host_work_requests(&instance)
            .must("pending requests should still be materializable")
            .into_iter()
            .find_map(|request| match request {
                qianji_bpmn_engine::PendingHostWorkRequest::Service(request)
                    if request.token_id == pending_work.token_id =>
                {
                    request.variables.get("item").cloned()
                }
                _ => None,
            })
            .must("parallel data-bound request should expose its current item");
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
                data: json!({
                    "result": format!("{}_done", item.as_str().must("item should be a string")),
                }),
            }),
            320,
        )
        .must("host completion should capture object-shaped data-binding output");
    }

    assert_eq!(
        instance.variables,
        json!({
            "assignments": {
                "alpha": "approve",
                "beta": "review",
            },
            "results": {
                "alpha": "approve_done",
                "beta": "review_done",
            }
        })
    );
    assert!(instance.variables.get("item").is_none());
    assert!(instance.variables.get("result").is_none());
}
