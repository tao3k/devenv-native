use super::super::{TRANSACTION_PROCESS_ID, node_index};
use super::helpers::{
    advance_and_expect_blocked, assert_main_success_completion, assert_pending_handler_node,
    complete_default_compensation_pair, complete_user_task_expect_advanced,
    create_transaction_test_instance,
};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    apply_pending_host_work_result,
};
use serde_json::json;

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_async_throw_compensation_intermediate_routes_forward_before_handler_completion()
 {
    let (package, mut instance, host) = create_transaction_test_instance(
        "transaction-throw-compensation-intermediate-async.bpmn",
        "wf_transaction_async_throw_compensation_intermediate",
    );
    let first_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the compensable activity",
    )
    .await;
    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        first_pending[0].token_id,
        json!({ "approved": true, "reviewer": "alice" }),
        100,
        "host completion should resume the transaction shell child",
    );

    let blocked_compensation = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("async throw compensation should route forward and then block on the handler");
    let compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        blocked_compensation,
        BpmnAdvanceOutcome::BlockedOnHost(compensation_pending.clone())
    );
    assert_pending_handler_node(
        &package,
        &compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_refund",
    );
    assert_eq!(
        instance.node_states[node_index(&package, TRANSACTION_PROCESS_ID, "tx_done") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
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
    .must("async compensation handler should finish after forward routing");
    assert_eq!(compensation_resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parent success path should complete after async compensation drains");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_main_success_completion(
        &package,
        &instance,
        &json!({ "amount": 7, "approved": true, "reviewer": "alice" }),
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_async_default_throw_compensation_intermediate_routes_forward_while_queue_drains()
 {
    let (package, mut instance, host) = create_transaction_test_instance(
        "transaction-default-compensation-intermediate-async.bpmn",
        "wf_transaction_async_default_throw_compensation_intermediate",
    );
    let expected_variables =
        complete_default_compensation_pair(&package, &mut instance, &host).await;

    let first_compensation_block = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "async default throw compensation should route forward and block on the first handler",
        );
    let first_compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        first_compensation_block,
        BpmnAdvanceOutcome::BlockedOnHost(first_compensation_pending.clone())
    );
    assert_pending_handler_node(
        &package,
        &first_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_capture",
    );
    assert_eq!(
        instance.node_states[node_index(&package, TRANSACTION_PROCESS_ID, "tx_done") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );

    complete_user_task_expect_advanced(
        package.as_ref(),
        &mut instance,
        first_compensation_pending[0].token_id,
        json!({ "released_capture": true }),
        150,
        "first async compensation handler should complete",
    );
    let second_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "detached async compensation should keep draining queued handlers after the first completion",
    )
    .await;
    assert_pending_handler_node(
        &package,
        &second_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_reserve",
    );
    assert_eq!(instance.variables, expected_variables);

    let second_compensation_resumed = apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        second_compensation_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "released_reserve": true }),
        }),
        180,
    )
    .must("second async compensation handler should drain the detached queue");
    assert_eq!(second_compensation_resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert_eq!(instance.variables, expected_variables);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parent success path should complete after async default compensation drains");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_main_success_completion(&package, &instance, &expected_variables);
}
