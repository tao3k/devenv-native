use crate::runtime::{StubHost, receive_task_wait_process};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnPackage, InstanceLifecycle, WaitKind,
    advance_instance, apply_event_poll_outcome, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_receive_task_registers_wait_and_resumes() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![receive_task_wait_process("receive_wait")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "receive_wait",
        BpmnInstanceInit::new("wf_receive_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("receive task should register one external wait");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 1);
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Message));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some("payment_received")
    );
    assert_eq!(
        instance.waits[0].event_name.as_deref(),
        Some("PaymentReceived")
    );
    assert_eq!(
        instance.node_states[1].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(instance.sequence, 3);

    let replay = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("registered receive-task wait should replay deterministically");
    assert_eq!(replay, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.sequence, 3);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        xiuxian_qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        91,
    )
    .must("ready receive-task message should resume");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.node_states[1].status,
        xiuxian_qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("resumed receive-task path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}
