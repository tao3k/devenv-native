use super::parse_fixture_package;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEventKind, BpmnNodeKind, BpmnTimerKind};

#[test]
fn parser_intermediate_message_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "intermediate-message-wait.bpmn",
        "bounded message wait should parse",
    );
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
    let package = parse_fixture_package(
        "intermediate-signal-wait.bpmn",
        "bounded signal wait should parse",
    );
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
    let package = parse_fixture_package(
        "intermediate-timer-wait.bpmn",
        "bounded timer wait should parse",
    );
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
