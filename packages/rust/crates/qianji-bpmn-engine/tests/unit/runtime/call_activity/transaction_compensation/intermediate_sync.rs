use super::helpers::{
    advance_and_expect_blocked, assert_main_success_completion, assert_pending_handler_node,
    complete_default_compensation_pair, complete_user_task_expect_advanced,
    create_transaction_test_instance,
};
use crate::runtime::call_activity::{TRANSACTION_PROCESS_ID, node_index};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_default_throw_compensation_intermediate_replays_all_handlers_then_routes_forward()
 {
    let (package, mut instance, host) = create_transaction_test_instance(
        "transaction-default-compensation-intermediate.bpmn",
        "wf_transaction_default_throw_compensation_intermediate",
    );
    let expected_variables =
        complete_default_compensation_pair(&package, &mut instance, &host).await;

    let first_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "default throw compensation intermediate event should start with the most recently completed handler",
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
        "first compensation handler should complete without mutating variables",
    );
    assert_eq!(instance.variables, expected_variables);

    let second_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "default throw compensation intermediate event should continue through remaining handlers in reverse order",
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
        "second compensation handler should complete without mutating variables",
    );
    assert_eq!(instance.variables, expected_variables);
    assert_eq!(
        instance.node_states[node_index(
            &package,
            TRANSACTION_PROCESS_ID,
            "tx_throw_intermediate_default",
        ) as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, TRANSACTION_PROCESS_ID, "tx_done") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "transaction shell should continue through the success path after default intermediate compensation",
        );
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_main_success_completion(&package, &instance, &expected_variables);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_throw_compensation_intermediate_runs_handler_then_routes_forward() {
    let package = Arc::new(crate::runtime::call_activity::parsed_fixture_package(
        "transaction-throw-compensation-intermediate.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_throw_compensation_intermediate",
            json!({ "amount": 7 }),
            10,
        ),
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
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );

    let blocked_compensation = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("throw compensation intermediate event should run the targeted handler before continuing");
    let compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        blocked_compensation,
        BpmnAdvanceOutcome::BlockedOnHost(compensation_pending.clone())
    );
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
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
    .must("targeted compensation handler should resume back into the transaction flow");
    assert_eq!(compensation_resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );
    assert_eq!(
        instance.node_states
            [node_index(&package, TRANSACTION_PROCESS_ID, "tx_throw_intermediate") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, TRANSACTION_PROCESS_ID, "tx_done") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Queued
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "transaction shell should continue through the normal success path after compensation",
        );
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_main_success_completion(
        &package,
        &instance,
        &json!({ "amount": 7, "approved": true, "reviewer": "alice" }),
    );
}
