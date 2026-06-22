use super::parse_fixture_package;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEventKind, BpmnNodeKind, BpmnTimerKind};

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

#[test]
fn parser_intermediate_conditional_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "intermediate-conditional-wait.bpmn",
        "bounded conditional wait should parse",
    );
    let process = package
        .find_process("await_condition")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::IntermediateCatchEvent);
    let event = process
        .event_for_node(1)
        .must("conditional wait should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Conditional);
    assert_eq!(event.name.as_deref(), Some("wait_condition"));
    assert_eq!(event.condition_expression.as_deref(), Some("approved"));
}

#[test]
fn parser_link_event_definition_materializes_metadata_event_binding() {
    let package = parse_fixture_package(
        "link-event-metadata.bpmn",
        "standard link events should parse as metadata events",
    );
    let process = package
        .find_process("link_event_metadata")
        .must("process should be present");

    let link_catch = process
        .event_for_node(1)
        .must("link catch should materialize an event binding");
    assert_eq!(link_catch.kind, BpmnEventKind::Link);
    assert_eq!(link_catch.name.as_deref(), Some("handoff"));

    let link_throw = process
        .event_for_node(2)
        .must("link throw should materialize an event binding");
    assert_eq!(link_throw.kind, BpmnEventKind::Link);
    assert_eq!(link_throw.name.as_deref(), Some("handoff"));
}

#[test]
fn parser_intermediate_throw_events_materialize_metadata_event_bindings() {
    let package = parse_fixture_package(
        "intermediate-throw-event-metadata.bpmn",
        "standard throw events should parse as metadata events",
    );
    let process = package
        .find_process("throw_event_metadata")
        .must("process should be present");

    let message_throw = process
        .event_for_node(1)
        .must("message throw should materialize an event binding");
    assert_eq!(message_throw.kind, BpmnEventKind::Message);
    assert_eq!(message_throw.reference_id.as_deref(), Some("notice"));

    let signal_throw = process
        .event_for_node(2)
        .must("signal throw should materialize an event binding");
    assert_eq!(signal_throw.kind, BpmnEventKind::Signal);
    assert_eq!(signal_throw.reference_id.as_deref(), Some("broadcast"));
}
