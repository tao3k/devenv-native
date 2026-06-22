use super::{StubHost, TRANSACTION_PROCESS_ID, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkResult,
    UserTaskOutcome, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_error_end_preserves_variables_and_routes_boundary_path() {
    let package = Arc::new(parsed_fixture_package("transaction-error-boundary.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_error", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);

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
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_error_end")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction error end should route the parent error boundary and finish");
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
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_boundary") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "errored_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_specific_error_routes_specific_and_catch_all_boundaries() {
    let package = Arc::new(parsed_fixture_package(
        "transaction-multi-error-boundaries.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_multi_error", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    let pending = instance.pending_host_work.clone();

    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));
    assert_eq!(pending.len(), 1);

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
    assert_eq!(instance.process.process_id.as_ref(), TRANSACTION_PROCESS_ID);
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_error_end")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction error end should route every matching parent error boundary and finish");
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
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_specific") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_cancel_boundary") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "cancelled_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_multi_error_ends_route_payment_specific_and_catch_all_boundaries() {
    let package = Arc::new(parsed_fixture_package("transaction-multi-error-ends.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_multi_error_ends_payment",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice", "payment_error": true }),
            }),
            100,
        )
        .must("host completion should resume the transaction shell child"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_error_route")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("payment error branch should route through the payment-specific and catch-all boundaries");
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice", "payment_error": true })
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_payment") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_fraud") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_errored_end") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "fraud_errored_end") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_multi_error_ends_route_default_fraud_boundary_and_catch_all() {
    let package = Arc::new(parsed_fixture_package("transaction-multi-error-ends.bpmn"));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new(
            "wf_transaction_multi_error_ends_fraud",
            json!({ "amount": 7 }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(55);

    let blocked = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction shell should enter the child process and block there");
    let pending = instance.pending_host_work.clone();
    assert_eq!(blocked, BpmnAdvanceOutcome::BlockedOnHost(pending.clone()));

    assert_eq!(
        crate::test_support::apply_pending_host_work_result(
            package.as_ref(),
            &mut instance,
            pending[0].token_id,
            PendingHostWorkResult::User(UserTaskOutcome {
                data: json!({ "approved": true, "reviewer": "alice", "payment_error": false }),
            }),
            100,
        )
        .must("host completion should resume the transaction shell child"),
        BpmnAdvanceOutcome::Advanced
    );
    assert_eq!(
        instance.active_tokens[0].node_index,
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_error_route")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must(
            "default fraud branch should route through the fraud-specific and catch-all boundaries",
        );
    assert_eq!(completion, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert_eq!(
        instance.variables,
        json!({ "amount": 7, "approved": true, "reviewer": "alice", "payment_error": false })
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_tx") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_fraud") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_payment") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "fraud_errored_end") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "payment_errored_end") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
