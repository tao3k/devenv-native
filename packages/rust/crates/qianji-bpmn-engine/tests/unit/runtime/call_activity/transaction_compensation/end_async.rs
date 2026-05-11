use super::helpers::{
    advance_and_expect_blocked, assert_main_success_completion, assert_pending_handler_node,
    complete_default_compensation_pair, complete_user_task, create_transaction_test_instance,
};
use crate::runtime::call_activity::TRANSACTION_PROCESS_ID;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnAdvanceOutcome, BpmnInstanceInit, create_instance};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_async_throw_compensation_end_resumes_parent_before_handler_completion()
{
    let package = Arc::new(crate::runtime::call_activity::parsed_fixture_package(
        "transaction-throw-compensation-end-async.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_async_throw_compensation_end",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = crate::runtime::call_activity::StubHost::new(55);

    let pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "transaction shell should block on the compensable activity",
    )
    .await;
    assert_eq!(
        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            qianji_bpmn_engine::PendingHostWorkResult::User(qianji_bpmn_engine::UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice" }),
            }),
            100,
        )
        .must("host completion should resume the transaction shell child"),
        BpmnAdvanceOutcome::Advanced
    );

    let compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "async throw compensation end event should let the parent scope finish while the handler remains pending",
    )
    .await;
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert_pending_handler_node(
        &package,
        &compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_refund",
    );

    assert_eq!(
        complete_user_task(
            package.as_ref(),
            &mut instance,
            compensation_pending[0].token_id,
            json!({ "refunded": true }),
            140,
            "targeted compensation handler should drain detached async completion",
        ),
        BpmnAdvanceOutcome::Advanced
    );
    assert_main_success_completion(
        &package,
        &instance,
        &json!({ "amount": 7, "approved": true, "reviewer": "alice" }),
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_async_default_throw_compensation_end_replays_handlers_after_parent_completion()
 {
    let (package, mut instance, host) = create_transaction_test_instance(
        "transaction-default-compensation-end-async.bpmn",
        "wf_transaction_async_default_throw_compensation_end",
    );
    let expected_variables =
        complete_default_compensation_pair(&package, &mut instance, &host).await;

    let first_compensation_pending = advance_and_expect_blocked(
        package.as_ref(),
        &mut instance,
        &host,
        "async default throw compensation end event should finish the parent scope while replay remains pending",
    )
    .await;
    assert_eq!(instance.process.process_id.as_ref(), "main_process");
    assert!(instance.call_stack.is_empty());
    assert_pending_handler_node(
        &package,
        &first_compensation_pending,
        TRANSACTION_PROCESS_ID,
        "tx_release_capture",
    );

    assert_eq!(
        complete_user_task(
            package.as_ref(),
            &mut instance,
            first_compensation_pending[0].token_id,
            json!({ "released_capture": true }),
            150,
            "first detached compensation handler should schedule the next handler",
        ),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(instance.variables, expected_variables);
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_pending_handler_node(
        &package,
        &instance.pending_host_work,
        TRANSACTION_PROCESS_ID,
        "tx_release_reserve",
    );
    let second_token_id = instance.pending_host_work[0].token_id;

    assert_eq!(
        complete_user_task(
            package.as_ref(),
            &mut instance,
            second_token_id,
            json!({ "released_reserve": true }),
            180,
            "second detached compensation handler should complete the instance",
        ),
        BpmnAdvanceOutcome::Advanced
    );
    assert_main_success_completion(&package, &instance, &expected_variables);
}
