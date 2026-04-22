use super::super::{StubHost, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnInstanceState, BpmnPackage, InstanceLifecycle,
    PendingHostWork, PendingHostWorkResult, UserTaskOutcome, apply_pending_host_work_result,
    create_instance,
};
use serde_json::json;
use std::sync::Arc;

pub(super) fn create_transaction_test_instance(
    fixture_name: &str,
    workflow_id: &str,
) -> (Arc<BpmnPackage>, BpmnInstanceState, StubHost) {
    let package = Arc::new(parsed_fixture_package(fixture_name));
    let instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(workflow_id, json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    (package, instance, StubHost::new(55))
}

pub(super) async fn advance_and_expect_blocked(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &StubHost,
    message: &str,
) -> Vec<PendingHostWork> {
    let blocked = qianji_bpmn_engine::advance_instance(package, instance, host)
        .await
        .must(message);
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    pending
}

pub(super) fn complete_user_task(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    token_id: u64,
    data: serde_json::Value,
    completed_at_ms: u64,
    message: &str,
) -> BpmnAdvanceOutcome {
    apply_pending_host_work_result(
        package,
        instance,
        token_id,
        PendingHostWorkResult::User(UserTaskOutcome { data }),
        completed_at_ms,
    )
    .must(message)
}

pub(super) fn complete_user_task_expect_advanced(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    token_id: u64,
    data: serde_json::Value,
    completed_at_ms: u64,
    message: &str,
) {
    assert_eq!(
        complete_user_task(package, instance, token_id, data, completed_at_ms, message,),
        BpmnAdvanceOutcome::Advanced
    );
}

pub(super) fn assert_pending_handler_node(
    package: &Arc<BpmnPackage>,
    pending: &[PendingHostWork],
    process_id: &str,
    node_id: &str,
) {
    assert_eq!(
        pending[0].node_index,
        node_index(package, process_id, node_id)
    );
}

pub(super) fn assert_main_success_completion(
    package: &Arc<BpmnPackage>,
    instance: &BpmnInstanceState,
    expected_variables: &serde_json::Value,
) {
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(instance.variables, *expected_variables);
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "payment_tx") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(package, "main_process", "success_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

pub(super) async fn complete_default_compensation_pair(
    package: &Arc<BpmnPackage>,
    instance: &mut BpmnInstanceState,
    host: &StubHost,
) -> serde_json::Value {
    let first_pending = advance_and_expect_blocked(
        package.as_ref(),
        instance,
        host,
        "transaction shell should block on the first compensable activity",
    )
    .await;
    complete_user_task_expect_advanced(
        package.as_ref(),
        instance,
        first_pending[0].token_id,
        json!({ "reserved": true }),
        90,
        "first activity should complete",
    );

    let second_pending = advance_and_expect_blocked(
        package.as_ref(),
        instance,
        host,
        "transaction shell should block on the second compensable activity",
    )
    .await;
    complete_user_task_expect_advanced(
        package.as_ref(),
        instance,
        second_pending[0].token_id,
        json!({ "captured": true }),
        120,
        "second activity should complete",
    );

    let expected_variables = json!({ "amount": 7, "reserved": true, "captured": true });
    assert_eq!(instance.variables, expected_variables);
    expected_variables
}
