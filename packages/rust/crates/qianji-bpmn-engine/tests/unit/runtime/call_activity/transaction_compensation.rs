use super::{StubHost, TRANSACTION_PROCESS_ID, node_index, parsed_fixture_package};
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
    let host = StubHost::new(55);

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
    assert_eq!(
        compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_refund")
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
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_cancel_boundary") as usize]
            .status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "cancelled_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_cancel_runs_compensation_in_reverse_completion_order() {
    let package = Arc::new(parsed_fixture_package(
        "transaction-cancel-compensation-reverse-order.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_compensation_reverse",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let first_block = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should block on the first compensable activity");
    let first_pending = instance.pending_host_work.clone();
    assert_eq!(
        first_block,
        BpmnAdvanceOutcome::BlockedOnHost(first_pending.clone())
    );
    apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        first_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "reserved": true }),
        }),
        90,
    )
    .must("first activity should complete");

    let second_block = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should block on the second compensable activity");
    let second_pending = instance.pending_host_work.clone();
    assert_eq!(
        second_block,
        BpmnAdvanceOutcome::BlockedOnHost(second_pending.clone())
    );
    apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        second_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "captured": true }),
        }),
        120,
    )
    .must("second activity should complete");

    let first_compensation_block = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("cancel path should start with the most recently completed compensation");
    let first_compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        first_compensation_block,
        BpmnAdvanceOutcome::BlockedOnHost(first_compensation_pending.clone())
    );
    assert_eq!(
        first_compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_release_capture")
    );
    apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        first_compensation_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "released_capture": true }),
        }),
        150,
    )
    .must("first compensation handler should complete");

    let second_compensation_block = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("remaining compensation should execute in reverse completion order");
    let second_compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        second_compensation_block,
        BpmnAdvanceOutcome::BlockedOnHost(second_compensation_pending.clone())
    );
    assert_eq!(
        second_compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_release_reserve")
    );
    apply_pending_host_work_result(
        package.as_ref(),
        &mut instance,
        second_compensation_pending[0].token_id,
        PendingHostWorkResult::User(UserTaskOutcome {
            data: json!({ "released_reserve": true }),
        }),
        180,
    )
    .must("second compensation handler should complete");

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("parent cancel boundary should route after the queue drains");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(instance.variables, json!({ "amount": 7 }));
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_throw_compensation_end_runs_targeted_handler_before_success_path() {
    let package = Arc::new(parsed_fixture_package(
        "transaction-throw-compensation-end.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_throw_compensation_end",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

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
        .must(
            "throw compensation end event should run the targeted handler before success routing",
        );
    let compensation_pending = instance.pending_host_work.clone();
    assert_eq!(
        blocked_compensation,
        BpmnAdvanceOutcome::BlockedOnHost(compensation_pending.clone())
    );
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(
        compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_refund")
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
    .must("targeted compensation handler should resume without mutating workflow variables");
    assert_eq!(compensation_resumed, BpmnAdvanceOutcome::Advanced);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice" })
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should complete through the success path after targeted compensation");
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
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}
