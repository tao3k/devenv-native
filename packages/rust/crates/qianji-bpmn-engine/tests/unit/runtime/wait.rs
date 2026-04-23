use super::{StubHost, intermediate_message_wait_process, intermediate_timer_wait_process};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnPackage, BpmnTimerKind,
    InstanceLifecycle, WaitKind, advance_instance, apply_event_poll_outcome, create_instance,
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test(flavor = "current_thread")]
async fn runtime_intermediate_message_event_registers_wait_and_resumes() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![intermediate_message_wait_process("message_wait")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "message_wait",
        BpmnInstanceInit::new("wf_message_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("intermediate message wait should register");

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
        qianji_bpmn_engine::NodeRuntimeStatus::Executing
    );
    assert_eq!(instance.sequence, 3);

    let replay = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("registered wait should replay deterministically");
    assert_eq!(replay, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.sequence, 3);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        91,
    )
    .must("ready message outcome should resume");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 2);
    assert_eq!(
        instance.node_states[1].status,
        qianji_bpmn_engine::NodeRuntimeStatus::Completed
    );
    assert_eq!(instance.variables, json!({ "amount": 7, "approved": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("resumed path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_intermediate_timer_event_registers_timer_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![intermediate_timer_wait_process("timer_wait")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "timer_wait",
        BpmnInstanceInit::new("wf_timer_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("intermediate timer wait should register");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 1);
    assert_eq!(instance.waits[0].blocking_node_index, None);
    assert_eq!(instance.waits[0].kind, WaitKind::Timer);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Timer));
    let timer = instance.waits[0]
        .timer
        .as_ref()
        .must("timer wait should preserve timer snapshot");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT5M");
}
