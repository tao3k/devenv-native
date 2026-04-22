use super::{StubHost, TRANSACTION_PROCESS_ID, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, BpmnInstanceState, BpmnPackage, InstanceLifecycle,
    PendingHostWork, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    apply_pending_host_work_result, create_instance,
};
use serde_json::json;
use std::sync::Arc;

fn create_transaction_test_instance(
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

async fn advance_and_expect_blocked(
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    host: &StubHost,
    message: &str,
) -> Vec<PendingHostWork> {
    let blocked = advance_instance(package, instance, host)
        .await
        .must(message);
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    pending
}

fn complete_user_task(
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

fn assert_main_success_completion(
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

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_default_throw_compensation_end_replays_all_handlers_before_success_path()
 {
    let (package, mut instance, host) = create_transaction_test_instance(
        "transaction-default-compensation-end.bpmn",
        "wf_transaction_default_throw_compensation_end",
    );
    let first_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the first compensable activity",
    )
    .await;
    let first_completion = complete_user_task(
        package.as_ref(),
        &mut instance,
        first_pending[0].token_id,
        json!({ "reserved": true }),
        90,
        "first activity should complete",
    );
    assert_eq!(first_completion, BpmnAdvanceOutcome::Advanced);

    let second_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the second compensable activity",
    )
    .await;
    let second_completion = complete_user_task(
        package.as_ref(),
        &mut instance,
        second_pending[0].token_id,
        json!({ "captured": true }),
        120,
        "second activity should complete",
    );
    assert_eq!(second_completion, BpmnAdvanceOutcome::Advanced);
    let expected_variables = json!({ "amount": 7, "reserved": true, "captured": true });
    assert_eq!(instance.variables, expected_variables);

    let first_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "default throw compensation should start with the most recently completed handler",
    )
    .await;
    assert_eq!(
        first_compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_release_capture")
    );
    let first_compensation_completion = complete_user_task(
        package.as_ref(),
        &mut instance,
        first_compensation_pending[0].token_id,
        json!({ "released_capture": true }),
        150,
        "first compensation handler should complete without mutating variables",
    );
    assert_eq!(first_compensation_completion, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.variables, expected_variables);

    let second_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "default throw compensation should continue through remaining handlers in reverse order",
    )
    .await;
    assert_eq!(
        second_compensation_pending[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_release_reserve")
    );
    let second_compensation_completion = complete_user_task(
        package.as_ref(),
        &mut instance,
        second_compensation_pending[0].token_id,
        json!({ "released_reserve": true }),
        180,
        "second compensation handler should complete without mutating variables",
    );
    assert_eq!(second_compensation_completion, BpmnAdvanceOutcome::Advanced);
    assert_eq!(instance.variables, expected_variables);

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "transaction shell should complete through the success path after default compensation",
        );
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_main_success_completion(&package, &instance, &expected_variables);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_throw_compensation_intermediate_runs_handler_then_routes_forward() {
    let package = Arc::new(parsed_fixture_package(
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
            "throw compensation intermediate event should run the targeted handler before continuing",
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
