use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnTimerKind};

#[test]
fn parser_start_event_message_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "start-message-wait.bpmn",
        "bounded message start wait should parse",
    );
    let process = package
        .find_process("start_message_wait")
        .must("process should be present");

    assert_eq!(process.nodes[0].kind, BpmnNodeKind::StartEvent);
    assert_eq!(process.events.len(), 1);
    let event = process
        .event_for_node(0)
        .must("start message binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("workflow_requested"));
    assert_eq!(event.name.as_deref(), Some("workflow_requested"));
}

#[test]
fn parser_start_event_signal_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "start-signal-wait.bpmn",
        "bounded signal start wait should parse",
    );
    let process = package
        .find_process("start_signal_wait")
        .must("process should be present");

    assert_eq!(process.nodes[0].kind, BpmnNodeKind::StartEvent);
    assert_eq!(process.events.len(), 1);
    let event = process
        .event_for_node(0)
        .must("start signal binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Signal);
    assert_eq!(event.reference_id.as_deref(), Some("workflow_signal"));
    assert_eq!(event.name.as_deref(), Some("workflow_signal"));
}

#[test]
fn parser_start_event_timer_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "start-timer-wait.bpmn",
        "bounded timer start wait should parse",
    );
    let process = package
        .find_process("start_timer_wait")
        .must("process should be present");

    assert_eq!(process.nodes[0].kind, BpmnNodeKind::StartEvent);
    let event = process
        .event_for_node(0)
        .must("start timer binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Timer);
    assert_eq!(event.name.as_deref(), Some("workflow_timer"));
    let timer = event.timer.as_ref().must("timer snapshot should exist");
    assert_eq!(timer.kind, BpmnTimerKind::Duration);
    assert_eq!(timer.expression.as_ref(), "PT5M");
}

#[test]
fn parser_start_event_conditional_wait_materializes_event_binding() {
    let package = parse_fixture_package(
        "start-conditional-wait.bpmn",
        "bounded conditional start wait should parse",
    );
    let process = package
        .find_process("start_conditional_wait")
        .must("process should be present");

    assert_eq!(process.nodes[0].kind, BpmnNodeKind::StartEvent);
    let event = process
        .event_for_node(0)
        .must("start conditional binding should be indexed by node");
    assert_eq!(event.kind, BpmnEventKind::Conditional);
    assert_eq!(event.name.as_deref(), Some("workflow_condition"));
    assert_eq!(event.condition_expression.as_deref(), Some("approved"));
}

#[test]
fn parser_escalation_deferred_start_event_reports_stable_detail() {
    let error = parse_fixture_error(
        "invalid-escalation-start-event.bpmn",
        "escalation start events should stay deferred with a stable detail",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: "escalation_start_event".to_string(),
            node_id: "start".to_string(),
            detail: "escalation_start_event_deferred",
        }
    );
}
