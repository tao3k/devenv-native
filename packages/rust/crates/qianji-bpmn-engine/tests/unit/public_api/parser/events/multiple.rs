use super::parse_fixture_error;
use qianji_bpmn_engine::BpmnEngineError;

#[test]
fn parser_multiple_event_definition_is_lint_deferred() {
    let error = parse_fixture_error(
        "invalid-multiple-event-definition.bpmn",
        "multiple event definitions should stay lint-deferred",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: "multiple_event_definition".to_string(),
            node_id: "wait_multiple".to_string(),
            detail: "multiple_event_definition_deferred",
        }
    );
}

#[test]
fn parser_parallel_multiple_event_definition_is_lint_deferred() {
    let error = parse_fixture_error(
        "invalid-parallel-multiple-event-definition.bpmn",
        "parallel multiple event definitions should stay lint-deferred",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: "parallel_multiple_event_definition".to_string(),
            node_id: "wait_parallel_multiple".to_string(),
            detail: "parallel_multiple_event_definition_deferred",
        }
    );
}

#[test]
fn parser_multiple_event_concrete_definitions_are_rejected() {
    let error = parse_fixture_error(
        "invalid-multiple-event-definitions.bpmn",
        "multiple concrete event definitions should be rejected",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedMultipleEventDefinitions {
            source_id: "invalid-multiple-event-definitions.bpmn".to_string(),
            process_id: "multiple_event_definitions".to_string(),
            node_id: "wait_multiple".to_string(),
        }
    );
}
