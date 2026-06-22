use super::StubHost;
use crate::test_support::MustExt as _;
use serde_json::json;
use std::sync::Arc;
use xiuxian_qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceInit, BpmnParseOptions, BpmnSourceFile,
    EventPollOutcome, InstanceLifecycle, WaitKind, advance_instance, apply_event_poll_outcome,
    build_event_poll_request, create_instance, parse_bpmn_package,
};

#[tokio::test(flavor = "current_thread")]
async fn runtime_interrupting_event_subprocess_message_cancels_parent_host_work() {
    let package = Arc::new(
        parse_bpmn_package(
            &[fixture_source("event-subprocess-message.bpmn")],
            &BpmnParseOptions::default(),
        )
        .must("event subprocess fixture should parse"),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_subprocess_message",
        BpmnInstanceInit::new("wf_event_subprocess_message", json!({ "amount": 7 }), 10),
    )
    .must("instance should be created");

    let blocked = advance_instance(package.as_ref(), &mut instance, &StubHost::new(11))
        .await
        .must("main path should block on host work while event subprocess is armed");
    let BpmnAdvanceOutcome::BlockedOnHost(pending) = blocked else {
        panic!("main path should block on service work");
    };
    assert_eq!(pending.len(), 1);
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].kind, WaitKind::ExternalEvent);
    assert_eq!(instance.waits[0].event_kind, Some(BpmnEventKind::Message));
    assert_eq!(
        instance.waits[0].event_reference.as_deref(),
        Some("interrupt_request")
    );

    let request = build_event_poll_request(&instance).must("event poll request should build");
    assert_eq!(request.waits.len(), 1);
    assert_eq!(request.waits[0].event_kind, Some(BpmnEventKind::Message));

    let triggered = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: true,
            winning_wait_node_index: None,
            data: json!({ "interrupted": true }),
        },
        21,
    )
    .must("ready event subprocess trigger should interrupt the parent scope");
    assert_eq!(triggered, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert!(instance.waits.is_empty());
    assert_eq!(
        instance.process.process_id.as_ref(),
        "__event_subprocess__::event_subprocess_message::interrupting_event_subprocess"
    );
    assert_eq!(instance.variables["interrupted"], json!(true));

    let completed = advance_instance(package.as_ref(), &mut instance, &StubHost::new(31))
        .await
        .must("event subprocess body should complete");
    assert_eq!(completed, BpmnAdvanceOutcome::Completed);
    assert_eq!(instance.lifecycle, InstanceLifecycle::Completed);
}

#[tokio::test(flavor = "current_thread")]
async fn runtime_conditional_event_subprocess_waits_until_condition_is_true() {
    let package = Arc::new(
        parse_bpmn_package(
            &[fixture_source("event-subprocess-conditional.bpmn")],
            &BpmnParseOptions::default(),
        )
        .must("conditional event subprocess fixture should parse"),
    );
    let mut instance = create_instance(
        Arc::clone(&package),
        "event_subprocess_conditional",
        BpmnInstanceInit::new("wf_event_subprocess_conditional", json!({}), 10),
    )
    .must("instance should be created");

    let blocked = advance_instance(package.as_ref(), &mut instance, &StubHost::new(11))
        .await
        .must("main path should block on host work while condition is armed");
    assert!(matches!(blocked, BpmnAdvanceOutcome::BlockedOnHost(_)));
    assert_eq!(instance.waits.len(), 1);
    assert_eq!(instance.waits[0].kind, WaitKind::Conditional);
    assert_eq!(
        instance.waits[0].condition_expression.as_deref(),
        Some("approved")
    );

    let still_waiting = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": false }),
        },
        21,
    )
    .must("false condition should keep the parent scope waiting");
    assert_eq!(still_waiting, BpmnAdvanceOutcome::WaitingExternalEvent);
    assert_eq!(instance.pending_host_work.len(), 1);
    assert_eq!(instance.waits.len(), 1);

    let triggered = apply_event_poll_outcome(
        package.as_ref(),
        &mut instance,
        EventPollOutcome {
            ready: false,
            winning_wait_node_index: None,
            data: json!({ "approved": true }),
        },
        31,
    )
    .must("true condition should trigger the event subprocess");
    assert_eq!(triggered, BpmnAdvanceOutcome::Advanced);
    assert!(instance.pending_host_work.is_empty());
    assert_eq!(
        instance.process.process_id.as_ref(),
        "__event_subprocess__::event_subprocess_conditional::interrupting_event_subprocess"
    );
}

fn fixture_source(name: &str) -> BpmnSourceFile {
    let path = format!("{}/tests/fixtures/bpmn/{name}", env!("CARGO_MANIFEST_DIR"));
    let contents =
        std::fs::read_to_string(path).must("fixture should be readable from the crate tree");
    BpmnSourceFile::new(name, contents)
}
