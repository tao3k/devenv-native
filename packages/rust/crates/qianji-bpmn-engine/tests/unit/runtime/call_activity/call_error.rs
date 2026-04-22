use super::{StubHost, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkResult,
    UserTaskOutcome, advance_instance, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_error_routes_specific_and_catch_all_boundaries() {
    let package = Arc::new(parsed_fixture_package("call-activity-error-boundary.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_call_activity_error", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": false, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the child process"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(instance.process.process_id.as_ref(), "child_process");
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, "child_process", "review_decision")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity error end should route through both matching parent error boundaries");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": false, "reviewer": "alice" })
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "invoke_review") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_specific") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_catch_all") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_success_cancels_error_boundaries() {
    let package = Arc::new(parsed_fixture_package("call-activity-error-boundary.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_call_activity_success", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the child process"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, "child_process", "review_decision")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should complete normally and cancel sibling error boundaries");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "invoke_review") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_specific") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_catch_all") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
