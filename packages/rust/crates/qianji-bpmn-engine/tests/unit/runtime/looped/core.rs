use crate::runtime::{StubHost, standard_loop_service_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, NodeRuntimeStatus, PendingHostWork,
    PendingHostWorkKind, PendingHostWorkResult, ServiceTaskOutcome, advance_instance,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_standard_loop_repeats_until_loop_maximum() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![standard_loop_service_process("loop_maximum")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "loop_maximum",
        BpmnInstanceInit::new("wf_loop_maximum", json!({ "done": false }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(101);

    for completed in 0..3 {
        let blocked = advance_instance(package.as_ref(), &mut instance, &host)
            .await
            .must("loop iteration should block on host work");
        let pending = instance
            .pending_host_work
            .first()
            .cloned()
            .must("loop iteration should register pending host work");
        assert_eq!(
            blocked,
            BpmnAdvanceOutcome::BlockedOnHost(vec![pending.clone()])
        );
        assert_eq!(
            pending,
            PendingHostWork {
                token_id: (instance.active_tokens[0].token_id),
                process_id: (Some("loop_maximum".into())),
                node_index: 1,
                activity_id: (Some("review".into())),
                kind: PendingHostWorkKind::Service,
                decision: None,
                lane: None,
                script_format: None,
                script_body: None,
                human_task_form: None,
                human_task_assignment: None,
                task_io: pending.task_io.clone(),
                claim: None,
                event_reference: None,
                event_name: None,
                work_id: None,
            }
        );
        assert_eq!(instance.standard_loops.len(), 1);
        assert_eq!(instance.standard_loops[0].completed_iterations, completed);

        let resumed = crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending.token_id,
            PendingHostWorkResult::Service(ServiceTaskOutcome { data: json!({}) }),
            120 + u64::from(completed),
        )
        .must("host completion should be applied");
        assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);

        if completed < 2 {
            assert_eq!(instance.active_tokens[0].node_index, 1);
            assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Queued);
            assert_eq!(
                instance.standard_loops[0].completed_iterations,
                completed + 1
            );
        } else {
            assert!(instance.standard_loops.is_empty());
            assert_eq!(instance.active_tokens[0].node_index, 2);
            assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
            assert_eq!(instance.node_states[2].status, NodeRuntimeStatus::Queued);
        }
    }

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("loop should complete after the final routed end event");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_standard_loop_skips_before_first_iteration_when_condition_is_false() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![standard_loop_service_process("loop_skip")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "loop_skip",
        BpmnInstanceInit::new("wf_loop_skip", json!({ "done": true }), 10),
    )
    .must("instance should be created");

    let outcome = advance_instance(package.as_ref(), &mut instance, &StubHost::new(140))
        .await
        .must("false pre-condition should skip the loop task");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.standard_loops.is_empty());
    assert_eq!(instance.node_states[1].status, NodeRuntimeStatus::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_standard_loop_stops_after_condition_turns_false() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![standard_loop_service_process("loop_condition")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "loop_condition",
        BpmnInstanceInit::new("wf_loop_condition", json!({ "done": false }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(155);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("first loop iteration should block");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    let token_id = instance.pending_host_work[0].token_id;

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::Service(ServiceTaskOutcome {
            data: json!({ "done": true }),
        }),
        166,
    )
    .must("loop completion should merge output and stop further iterations");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.standard_loops.is_empty());
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.variables, json!({ "done": true }));

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("loop should complete once the condition turns false");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
}
