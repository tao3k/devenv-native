use super::{EMBEDDED_REVIEW_PROCESS_ID, StubHost, TRANSACTION_PROCESS_ID, parsed_fixture_package};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkKind,
    PendingHostWorkResult, UserTaskOutcome, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_embedded_subprocess_uses_existing_child_process_frame_model() {
    let package = Arc::new(parsed_fixture_package("embedded-subprocess-basic.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_embedded_subprocess", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess should enter the child process and block there");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, PendingHostWorkKind::User);
    assert_eq!(pending[0].node_index, 1);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(
        instance.process.process_id.as_ref(),
        EMBEDDED_REVIEW_PROCESS_ID
    );
    assert_eq!(instance.call_stack.len(), 1);
    assert_eq!(
        instance.call_stack[0].process.process_id.as_ref(),
        "main_process"
    );
    assert_eq!(instance.call_stack[0].return_node_index, 1);

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("host completion should resume the embedded subprocess child");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(
        instance.process.process_id.as_ref(),
        EMBEDDED_REVIEW_PROCESS_ID
    );
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.call_stack.len(), 1);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("embedded subprocess completion should restore the parent process and finish");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[1].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_shell_uses_existing_child_process_frame_model() {
    let package = Arc::new(parsed_fixture_package("transaction-basic.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_shell", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, PendingHostWorkKind::User);
    assert_eq!(pending[0].node_index, 1);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(instance.call_stack.len(), 1);
    assert_eq!(
        instance.call_stack[0].process.process_id.as_ref(),
        "main_process"
    );
    assert_eq!(instance.call_stack[0].return_node_index, 1);

    let resumed = crate::test_support::apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("host completion should resume the transaction shell child");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.call_stack.len(), 1);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell completion should restore the parent process and finish");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[1].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}
