use super::super::super::fixture_source;
use super::EMBEDDED_REVIEW_PROCESS_ID;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package,
};

#[test]
fn parser_embedded_subprocess_materializes_synthetic_child_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("embedded-subprocess-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded embedded subprocess should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some(EMBEDDED_REVIEW_PROCESS_ID)
    );

    let child = package
        .find_process(EMBEDDED_REVIEW_PROCESS_ID)
        .must("embedded subprocess child process should be present");
    assert_eq!(child.nodes.len(), 3);
    assert_eq!(child.nodes[1].kind, BpmnNodeKind::UserTask);
    assert_eq!(child.nodes[1].bpmn_id.as_ref(), "sub_review");
}

#[test]
fn parser_embedded_subprocess_requires_exactly_one_start_event() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-embedded-subprocess-multiple-starts.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("embedded subprocess should reject multiple nested start events");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: "main_process".to_string(),
            node_id: "inline_review".to_string(),
            detail: "embedded_subprocess_start_event_count",
        }
    );
}

#[test]
fn parser_embedded_subprocess_error_path_materializes_parent_boundaries_and_child_error_end() {
    let package = parse_bpmn_package(
        &[fixture_source("embedded-subprocess-error-boundary.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("embedded subprocess error boundary should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some(EMBEDDED_REVIEW_PROCESS_ID)
    );
    assert_eq!(
        process
            .boundary_events_for_attached_node(process.nodes[1].index)
            .count(),
        2
    );

    let child = package
        .find_process(EMBEDDED_REVIEW_PROCESS_ID)
        .must("embedded subprocess child process should be present");
    let error_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "sub_error_end")
        .must("embedded subprocess error end should be present");
    assert_eq!(error_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(error_end.index)
        .must("embedded subprocess error end should expose event metadata");
    assert_eq!(event.kind, BpmnEventKind::Error);
    assert_eq!(event.reference_id.as_deref(), Some("review_rejected"));
}

#[test]
fn parser_embedded_subprocess_error_path_requires_matching_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-embedded-subprocess-error-missing-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("embedded subprocess error path should reject missing parent boundaries");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: "main_process".to_string(),
            node_id: "inline_review".to_string(),
            detail: "embedded_subprocess_error_missing_boundary",
        }
    );
}

#[test]
fn parser_embedded_subprocess_interrupting_external_boundaries_accept_timer_message_and_signal() {
    for (fixture_name, boundary_id, event_kind) in [
        (
            "embedded-subprocess-timer-boundary.bpmn",
            "review_timeout",
            BpmnEventKind::Timer,
        ),
        (
            "embedded-subprocess-message-boundary.bpmn",
            "review_escalated",
            BpmnEventKind::Message,
        ),
        (
            "embedded-subprocess-signal-boundary.bpmn",
            "review_alert",
            BpmnEventKind::Signal,
        ),
    ] {
        let package = parse_bpmn_package(
            &[fixture_source(fixture_name)],
            &BpmnParseOptions::default(),
        )
        .must("embedded subprocess interrupting external boundary should parse");
        let process = package
            .find_process("main_process")
            .must("main process should be present");

        assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
        assert_eq!(
            process.nodes[1].called_process_id.as_deref(),
            Some(EMBEDDED_REVIEW_PROCESS_ID)
        );
        let boundary = process
            .nodes
            .iter()
            .find(|node| node.bpmn_id.as_ref() == boundary_id)
            .must("embedded subprocess boundary should be present");
        assert_eq!(boundary.kind, BpmnNodeKind::BoundaryEvent);
        assert!(boundary.cancel_activity);
        assert_eq!(
            process
                .event_for_node(boundary.index)
                .must("embedded subprocess boundary should expose event metadata")
                .kind,
            event_kind
        );
    }
}

#[test]
fn parser_embedded_subprocess_accepts_mixed_external_and_error_boundaries() {
    let package = parse_bpmn_package(
        &[fixture_source("embedded-subprocess-mixed-boundaries.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("embedded subprocess mixed boundaries should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some(EMBEDDED_REVIEW_PROCESS_ID)
    );
    assert_eq!(
        process
            .boundary_events_for_attached_node(process.nodes[1].index)
            .count(),
        3
    );
}
