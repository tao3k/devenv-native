use super::super::super::{
    StubHost, sequential_multi_instance_process,
    sequential_multi_instance_process_with_completion_condition,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnNodeKind, BpmnPackage, PendingHostWork,
    PendingHostWorkKind, PendingHostWorkResult, ServiceTaskOutcome, advance_instance,
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
                process_id: Some("multi_instance_service".to_string()),
                node_index: 1,
                kind: PendingHostWorkKind::Service,
                decision: None,
                script_format: None,
                script_body: None,
                event_reference: None,
                event_name: None,
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
