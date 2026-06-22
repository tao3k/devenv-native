use super::helpers::{
    advance_and_expect_blocked, assert_main_success_completion, assert_pending_handler_node,
    complete_user_task, create_transaction_test_instance,
};
use crate::runtime::call_activity::TRANSACTION_PROCESS_ID;
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, PendingHostWorkResult, UserTaskOutcome, advance_instance,
    create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_throw_compensation_end_runs_targeted_handler_before_success_path() {
    let package = Arc::new(crate::runtime::call_activity::parsed_fixture_package(
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
    let host = crate::runtime::call_activity::StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should block on the compensable activity");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    let resumed = crate::test_support::apply_pending_host_work_result(
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
    assert_pending_handler_node(
        &package,
        &compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_refund",
    );

    let compensation_resumed = crate::test_support::apply_pending_host_work_result(
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
    assert_main_success_completion(
        &package,
        &instance,
        &json!({ "amount": 7, "approved": true, "reviewer": "alice" }),
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
    assert_pending_handler_node(
        &package,
        &first_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_capture",
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
    assert_pending_handler_node(
        &package,
        &second_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_reserve",
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
