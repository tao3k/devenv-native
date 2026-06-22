use super::{StubHost, TRANSACTION_PROCESS_ID, node_index, parsed_fixture_package};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnInstanceInit, InstanceLifecycle, PendingHostWorkResult,
    UserTaskOutcome, advance_instance, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_transaction_normal_completion_cancels_all_boundaries() {
    let package = Arc::new(parsed_fixture_package(
        "transaction-multi-complete-boundaries.bpmn",
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "main_process",
        BpmnInstanceInit::new("wf_transaction_multi_complete", json!({ "amount": 7 }), 10),
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
        node_index(&package, TRANSACTION_PROCESS_ID, "tx_end")
    );

    let completion = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("transaction normal completion should restore the parent process and finish");
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
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_cancel_boundary") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_specific") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "tx_error_catch_all") as usize]
            .status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Cancelled
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "success_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "cancelled_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "specific_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
    assert_eq!(
        instance.node_states[node_index(&package, "main_process", "catch_all_end") as usize].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Idle
    );
}
