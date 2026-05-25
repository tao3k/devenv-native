use crate::public_api::parser::events::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnTimerKind};

#[test]
fn parser_non_interrupting_boundary_timer_materializes_attachment_and_timer_snapshot() {
    let package = parse_fixture_package(
        "boundary-timer-non-interrupt.bpmn",
        "bounded non-interrupting boundary timer should parse",
    );
    let process = package
        .find_process("review_with_timeout")
        .must("process should be present");

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
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
fn parser_non_interrupting_boundary_message_materializes_attachment_and_event_binding() {
    let package = parse_fixture_package(
        "boundary-message-non-interrupt.bpmn",
        "bounded non-interrupting boundary message should parse",
    );
    let process = package
        .find_process("review_with_message_watch")
        .must("process should be present");

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("boundary message should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("review_message"));
    assert_eq!(event.name.as_deref(), Some("review_escalated"));
}

#[test]
fn parser_non_interrupting_boundary_signal_materializes_attachment_and_event_binding() {
    let package = parse_fixture_package(
        "boundary-signal-non-interrupt.bpmn",
        "bounded non-interrupting boundary signal should parse",
    );
    let process = package
        .find_process("review_with_signal_watch")
        .must("process should be present");

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("boundary signal should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Signal);
    assert_eq!(event.reference_id.as_deref(), Some("review_signal"));
    assert_eq!(event.name.as_deref(), Some("review_alert"));
}

#[test]
fn parser_non_interrupting_conditional_boundary_materializes_attachment_and_condition() {
    let package = parse_fixture_package(
        "boundary-conditional-non-interrupt.bpmn",
        "bounded non-interrupting boundary conditional event should parse",
    );
    let process = package
        .find_process("review_with_conditional_watch")
        .must("process should be present");

    assert_eq!(process.nodes[2].kind, BpmnNodeKind::BoundaryEvent);
    assert_eq!(process.nodes[2].attached_to, Some(1));
    assert!(!process.nodes[2].cancel_activity);
    let boundary = process
        .boundary_event_for_attached_node(1)
        .must("attached task should resolve the boundary event");
    assert_eq!(boundary.index, 2);
    let event = process
        .event_for_node(boundary.index)
        .must("boundary conditional should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Conditional);
    assert_eq!(event.condition_expression.as_deref(), Some("escalated"));
}

#[test]
fn parser_escalation_deferred_non_interrupting_boundary_reports_stable_detail() {
    let error = parse_fixture_error(
        "invalid-boundary-escalation-non-interrupt.bpmn",
        "non-interrupting escalation boundaries should stay deferred with a stable detail",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedBoundaryEventConfiguration {
            process_id: ("review_with_non_interrupting_escalation".to_string()).into(),
            node_id: ("review_escalated".to_string()).into(),
            detail: "non_interrupting_escalation_boundary_deferred",
        }
    );
}
