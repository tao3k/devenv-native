use super::helpers::{
    advance_and_expect_blocked, assert_pending_handler_node, complete_user_task_expect_advanced,
};
use crate::runtime::call_activity::{TRANSACTION_PROCESS_ID, parsed_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkResult,
    UserTaskOutcome, advance_instance, apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_cancel_runs_compensation_before_boundary_path() {
    let package = Arc::new(parsed_fixture_package(
        "transaction-cancel-compensation.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_compensation", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = crate::runtime::call_activity::StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should block on the compensable activity");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    let resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "approved": true, "reviewer": "alice" }),
        }),
        100,
    )
    .must("host completion should resume the transaction shell child");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);

    let blocked_compensation = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction cancel should run compensation before boundary routing");
    let compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        blocked_compensation,
        BpmnAdvanceOutcome::BlockedOnHost(compensation_pending.clone())
    );
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
    assert_pending_handler_node(
        &package,
        &compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_refund",
    );

    let compensation_resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        compensation_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "refunded": true }),
        }),
        140,
    )
    .must("compensation handler should resume without mutating workflow variables");
    assert_eq!(compensation_resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.variables, json!({ "amount": 7 }));

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction cancel should route the parent boundary after compensation");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert!(instance.active_tokens.is_empty());
    assert_eq!(instance.variables, json!({ "amount": 7 }));
    assert_eq!(
        instance.node_states[crate::runtime::call_activity::node_index(
            &package,
            "main_process",
            "payment_tx"
        ) as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[crate::runtime::call_activity::node_index(
            &package,
            "main_process",
            "tx_cancel_boundary"
        ) as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[crate::runtime::call_activity::node_index(
            &package,
            "main_process",
            "cancelled_end"
        ) as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_cancel_runs_compensation_in_reverse_completion_order() {
    let (package, mut instance, host) = super::helpers::create_transaction_test_instance(
        "transaction-cancel-compensation-reverse-order.bpmn",
        "wf_transaction_compensation_reverse",
    );
    let first_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the first compensable activity",
    )
    .await;
    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        first_pending[0].token_id,
        json!({ "reserved": true }),
        90,
        "first activity should complete",
    );

    let second_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the second compensable activity",
    )
    .await;
    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        second_pending[0].token_id,
        json!({ "captured": true }),
        120,
        "second activity should complete",
    );

    let first_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "cancel path should start with the most recently completed compensation",
    )
    .await;
    assert_pending_handler_node(
        &package,
        &first_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_capture",
    );
    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        first_compensation_pending[0].token_id,
        json!({ "released_capture": true }),
        150,
        "first compensation handler should complete",
    );

    let second_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "remaining compensation should execute in reverse completion order",
    )
    .await;
    assert_pending_handler_node(
        &package,
        &second_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_reserve",
    );
    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        second_compensation_pending[0].token_id,
        json!({ "released_reserve": true }),
        180,
        "second compensation handler should complete",
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parent cancel boundary should route after the queue drains");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
}
