use crate::runtime::{
    StubHost, parallel_multi_instance_process,
    parallel_multi_instance_process_with_completion_condition,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, ServiceTaskOutcome,
    advance_instance, create_instance,
};

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
        let resumed = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending_work.token_id,
            xiuxian_qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
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
                xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
            );
            assert_eq!(
                instance.node_states[2].status,
                xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Queued
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
        xiuxian_qianji_bpmn_engine::InstanceLifecycle::Completed
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

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[1].token_id,
        xiuxian_qianji_bpmn_engine::PendingHostWorkResult::Service(ServiceTaskOutcome {
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
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "winner": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("completion-condition parallel path should finish");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}
