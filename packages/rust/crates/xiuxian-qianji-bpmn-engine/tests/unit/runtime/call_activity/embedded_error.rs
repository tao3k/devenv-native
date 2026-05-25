use super::{EMBEDDED_REVIEW_PROCESS_ID, StubHost, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkResult,
    UserTaskOutcome, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_embedded_subprocess_error_routes_specific_and_catch_all_boundaries() {
    let package = Arc::new(parsed_fixture_package(
        "embedded-subprocess-error-boundary.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_embedded_subprocess_error", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": false, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the embedded subprocess child"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(
        instance.process.process_id.as_ref(),
        EMBEDDED_REVIEW_PROCESS_ID
    );
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, EMBEDDED_REVIEW_PROCESS_ID, "review_decision")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess error end should route through both matching parent error boundaries");
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
        instance.node_states[node_index(&package, "main_process", "inline_review") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_specific") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_embedded_subprocess_success_cancels_error_boundaries() {
    let package = Arc::new(parsed_fixture_package(
        "embedded-subprocess-error-boundary.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_embedded_subprocess_success", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the embedded subprocess child"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, EMBEDDED_REVIEW_PROCESS_ID, "review_decision")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess should complete normally and cancel sibling error boundaries");
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
        instance.node_states[node_index(&package, "main_process", "inline_review") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_specific") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, "main_process", "review_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
