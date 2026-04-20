use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnTimerKind,
    parse_bpmn_package,
};

#[test]
fn parser_intermediate_message_wait_materializes_event_binding() {
    let package = parse_bpmn_package(
        &[fixture_source("intermediate-message-wait.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded message wait should parse");
    let process = package
        .find_process("await_message")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::IntermediateCatchEvent);
    assert_eq!(process.events.len(), 1);
    let event = process
        .event_for_node(1)
        .must("event binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("payment_received"));
    assert_eq!(event.name.as_deref(), Some("wait_message"));
}

#[test]
fn parser_intermediate_signal_wait_materializes_event_binding() {
    let package = parse_bpmn_package(
        &[fixture_source("intermediate-signal-wait.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded signal wait should parse");
    let process = package
        .find_process("await_signal")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::IntermediateCatchEvent);
    assert_eq!(process.events.len(), 1);
    let event = process
        .event_for_node(1)
        .must("event binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Signal);
    assert_eq!(event.reference_id.as_deref(), Some("alert_signal"));
    assert_eq!(event.name.as_deref(), Some("wait_signal"));
}

#[test]
fn parser_intermediate_timer_wait_materializes_event_binding() {
    let package = parse_bpmn_package(
        &[fixture_source("intermediate-timer-wait.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded timer wait should parse");
    let process = package
        .find_process("await_timer")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::IntermediateCatchEvent);
    let event = process
        .event_for_node(1)
        .must("timer wait should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    assert_eq!(event.name.as_deref(), Some("wait_timer"));
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT5M");
}

#[test]
fn parser_boundary_timer_interrupt_materializes_attachment_and_timer_snapshot() {
    let package = parse_bpmn_package(
        &[fixture_source("boundary-timer-interrupt.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("interrupting boundary timer should parse");
    let process = package
        .find_process("review_with_timeout")
        .must("process should be present");

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("boundary timer should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    assert_eq!(event.name.as_deref(), Some("review_timeout"));
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT30M");
}

#[test]
fn parser_intermediate_catch_event_requires_event_definition() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-intermediate-catch-missing-event-definition.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("intermediate catch event without event definition should fail");

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: "await_missing_event".to_string(),
            node_id: "wait_missing".to_string(),
            element: "event_definition",
        }
    );
}

#[test]
fn parser_intermediate_timer_event_requires_timer_expression() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-intermediate-timer-event.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("timer waits without an expression should fail validation");

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: "await_timer".to_string(),
            node_id: "wait_timer".to_string(),
            element: "timer_expression",
        }
    );
}

#[test]
fn parser_non_interrupting_boundary_timer_is_rejected() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-non-interrupting-boundary-timer.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("non-interrupting boundary timers remain outside the bounded slice");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: "review_with_timeout".to_string(),
            node_id: "review_timeout".to_string(),
            detail: "non_interrupting_boundary_event",
        }
    );
}
