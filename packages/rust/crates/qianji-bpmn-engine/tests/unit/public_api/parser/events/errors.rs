use super::parse_fixture_error;
use super::parse_fixture_package;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnEventKind, BpmnNodeKind};

#[test]
fn parser_top_level_error_end_materializes_event_binding() {
    let package = parse_fixture_package(
        "top-level-error-end.bpmn",
        "top-level error end should parse",
    );
    let process = package
        .find_process("root_error_process")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::EndEvent);
    let event = process
        .event_for_node(1)
        .must("top-level error end should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Error);
    assert_eq!(event.reference_id.as_deref(), Some("fatal_review_error"));
}

#[test]
fn parser_intermediate_catch_event_requires_event_definition() {
    let error = parse_fixture_error(
        "invalid-intermediate-catch-missing-event-definition.bpmn",
        "intermediate catch event without event definition should fail",
    );

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: ("await_missing_event".to_string()).into(),
            node_id: ("wait_missing".to_string()).into(),
            element: "event_definition",
        }
    );
}

#[test]
fn parser_intermediate_timer_event_requires_timer_expression() {
    let error = parse_fixture_error(
        "invalid-intermediate-timer-event.bpmn",
        "timer waits without an expression should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: ("await_timer".to_string()).into(),
            node_id: ("wait_timer".to_string()).into(),
            element: "timer_expression",
        }
    );
}

#[test]
fn parser_intermediate_conditional_event_requires_condition_expression() {
    let error = parse_fixture_error(
        "invalid-intermediate-conditional-missing-condition.bpmn",
        "conditional waits without a condition should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: ("await_condition".to_string()).into(),
            node_id: ("wait_condition".to_string()).into(),
            element: "conditional_expression",
        }
    );
}

#[test]
fn parser_intermediate_conditional_event_rejects_unsupported_condition_expression() {
    let error = parse_fixture_error(
        "invalid-intermediate-conditional-unsupported-condition.bpmn",
        "unsupported conditional wait expression should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: ("await_condition".to_string()).into(),
            node_id: ("wait_condition".to_string()).into(),
            detail: "unsupported_conditional_event_expression",
        }
    );
}

#[test]
fn parser_conditional_boundary_event_requires_condition_expression() {
    let error = parse_fixture_error(
        "invalid-boundary-conditional-missing-condition.bpmn",
        "conditional boundary events without a condition should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: ("review_with_invalid_conditional_boundary".to_string()).into(),
            node_id: ("review_condition".to_string()).into(),
            element: "conditional_expression",
        }
    );
}

#[test]
fn parser_conditional_boundary_event_rejects_unsupported_condition_expression() {
    let error = parse_fixture_error(
        "invalid-boundary-conditional-unsupported-condition.bpmn",
        "unsupported conditional boundary expression should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: ("review_with_invalid_conditional_boundary".to_string()).into(),
            node_id: ("review_condition".to_string()).into(),
            detail: "unsupported_conditional_event_expression",
        }
    );
}
