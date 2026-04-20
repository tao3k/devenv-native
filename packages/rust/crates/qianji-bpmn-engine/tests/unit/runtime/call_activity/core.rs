use super::{StubHost, call_activity_child_process, call_activity_main_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnPackage, InstanceLifecycle, PendingHostWorkKind,
    PendingHostWorkResult, UserTaskOutcome, advance_instance, apply_pending_host_work_result,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_enters_child_process_and_blocks_inside_child() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![
            call_activity_main_process("main_process"),
            call_activity_child_process(),
        ],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_call_activity", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, PendingHostWorkKind::User);
    assert_eq!(pending[0].node_index, 1);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.process.process_id.as_ref(), "child_process");
    assert_eq!(instance.call_stack.len(), 1);
    assert_eq!(
        instance.call_stack[0].process.process_id.as_ref(),
        "main_process"
    );
    assert_eq!(instance.call_stack[0].return_node_index, 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(
        instance.call_stack[0].node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_call_activity_host_completion_returns_to_parent_process() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![
            call_activity_main_process("main_process"),
            call_activity_child_process(),
        ],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_call_activity", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("call activity should enter the child process and block there");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    let token_id = instance.pending_host_work[0].token_id;

    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true }),
        }),
        100,
    )
    .must("host completion should resume the child process");

    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.process.process_id.as_ref(), "child_process");
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(instance.call_stack.len(), 1);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("child completion should restore the parent process and finish");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));
}
