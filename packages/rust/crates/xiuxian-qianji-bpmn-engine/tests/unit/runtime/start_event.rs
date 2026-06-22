use super::{
    StubHost, start_event_conditional_wait_process, start_event_message_wait_process,
    start_event_signal_wait_process, start_event_timer_wait_process,
};
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnPackage, BpmnTimerKind,
    InstanceLifecycle, NodeRuntimeStatus, WaitKind, advance_instance, apply_event_poll_outcome,
    build_event_poll_request, create_instance,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_message_start_event_registers_wait_and_resumes() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![start_event_message_wait_process("message_start")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "message_start",
        BpmnInstanceInit::new("wf_message_start", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("message start event should register");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.active_tokens.len(), 1);
    assert_eq!(instance.active_tokens[0].node_index, 0);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 0);
    assert_eq!(instance.waits[0].blocking_node_index, None);
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Message));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some("workflow_requested")
    );
    assert_eq!(
        instance.waits[0].event_name.as_deref(),
        Some("WorkflowRequested")
    );
    assert_eq!(instance.node_states[0].status, NodeRuntimeStatus::Executing);

    let request = build_event_poll_request(&instance).must("poll request should build");
    assert_eq!(request.waits.len(), 1);
    assert_eq!(request.waits[0].node_index, 0);

    let replay = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("registered start wait should replay deterministically");
    assert_eq!(replay, BpmnAdvanceOutcome::WaitingExternalEvent);

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        xiuxian_qianji_bpmn_engine::EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "started": true }),
        },
        91,
    )
    .must("ready message start outcome should resume");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 1);
    assert_eq!(instance.node_states[0].status, NodeRuntimeStatus::Completed);
    assert_eq!(instance.variables, json!({ "amount": 7, "started": true }));

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("resumed start path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_signal_start_event_registers_external_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![start_event_signal_wait_process("signal_start")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "signal_start",
        BpmnInstanceInit::new("wf_signal_start", json!({}), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("signal start event should register");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 0);
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Signal));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some("workflow_signal")
    );
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_timer_start_event_registers_timer_wait() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![start_event_timer_wait_process("timer_start")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "timer_start",
        BpmnInstanceInit::new("wf_timer_start", json!({}), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("timer start event should register");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 0);
    assert_eq!(instance.waits[0].kind, WaitKind::Timer);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Timer));
    let timer = instance.waits[0]
        .timer
        .as_ref()
        .must("timer wait should preserve timer snapshot");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT5M");
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_conditional_start_event_routes_immediately_when_condition_is_true() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![start_event_conditional_wait_process("conditional_start")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "conditional_start",
        BpmnInstanceInit::new(
            "wf_conditional_start_ready",
            json!({ "approved": true }),
            10,
        ),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("ready conditional start should route");

    assert_eq!(outcome, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.node_states[0].status, NodeRuntimeStatus::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_conditional_start_event_waits_and_resumes_when_condition_becomes_true() {
    let package = Arc::new(BpmnPackage::new(
        "pkg_runtime",
        vec![start_event_conditional_wait_process("conditional_start")],
    ));
    let mut instance = create_instance(
        Arc::clone(&package),
        "conditional_start",
        BpmnInstanceInit::new("wf_conditional_start_wait", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");
    let host = StubHost::new(77);

    let outcome = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("unready conditional start should register");

    assert_eq!(outcome, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Waiting);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].node_index, 0);
    assert_eq!(instance.waits[0].kind, WaitKind::Conditional);
    assert_eq!(
        instance.waits[0].event_kind,
        Some(BpmnEventKind::Conditional)
    );
    assert_eq!(
        instance.waits[0].condition_expression.as_deref(),
        Some("approved")
    );

    let still_waiting = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        xiuxian_qianji_bpmn_engine::EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": false }),
        },
        91,
    )
    .must("false conditional start poll should stay waiting");
    assert_eq!(still_waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.variables["approved"], json!(false));

    let resumed = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        xiuxian_qianji_bpmn_engine::EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        101,
    )
    .must("true conditional start poll should resume");
    assert_eq!(resumed, BpmnAdvanceOutcome::Advanced);
    assert!(instance.waits.is_empty());
    assert_eq!(instance.lifecycle, InstanceLifecycle::Running);
    assert_eq!(instance.active_tokens[0].node_index, 1);

    let completed = advance_instance(package.as_ref(), &mut instance, &host)
        .await
        .must("resumed conditional start path should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
}
