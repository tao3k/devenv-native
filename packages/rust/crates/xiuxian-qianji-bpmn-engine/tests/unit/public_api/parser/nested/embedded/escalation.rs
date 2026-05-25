use super::{EMBEDDED_REVIEW_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package,
};

#[test]
fn parser_embedded_subprocess_escalation_path_materializes_parent_boundaries_and_child_end() {
    let package = parse_bpmn_package(
        &[fixture_source(
            "embedded-subprocess-escalation-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must("embedded subprocess escalation boundary should parse");
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
    let escalation_end = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "sub_escalation_end")
        .must("embedded subprocess escalation end should be present");
    assert_eq!(escalation_end.kind, BpmnNodeKind::EndEvent);
    let event = child
        .event_for_node(escalation_end.index)
        .must("embedded subprocess escalation end should expose event metadata");
    assert_eq!(event.kind, BpmnEventKind::Escalation);
    assert_eq!(event.reference_id.as_deref(), Some("review_escalated"));
}

#[test]
fn parser_embedded_subprocess_escalation_path_requires_matching_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-embedded-subprocess-escalation-missing-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("embedded subprocess escalation path should reject missing parent boundaries");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("inline_review".to_string()).into(),
            detail: "embedded_subprocess_escalation_missing_boundary",
        }
    );
}

#[test]
fn parser_embedded_subprocess_intermediate_escalation_materializes_throw_event() {
    let package = parse_bpmn_package(
        &[fixture_source(
            "embedded-subprocess-intermediate-escalation-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must("embedded subprocess intermediate escalation boundary should parse");
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
    let escalation_throw = child
        .nodes
        .iter()
        .find(|node| node.bpmn_id.as_ref() == "sub_escalation_throw")
        .must("embedded subprocess escalation throw should be present");
    assert_eq!(escalation_throw.kind, BpmnNodeKind::IntermediateThrowEvent);
    let event = child
        .event_for_node(escalation_throw.index)
        .must("embedded subprocess escalation throw should expose event metadata");
    assert_eq!(event.kind, BpmnEventKind::Escalation);
    assert_eq!(event.reference_id.as_deref(), Some("review_escalated"));
}

#[test]
fn parser_embedded_subprocess_intermediate_escalation_requires_matching_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source(
            "invalid-embedded-subprocess-intermediate-escalation-missing-boundary.bpmn",
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("embedded subprocess intermediate escalation should reject missing parent boundary");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("inline_review".to_string()).into(),
            detail: "embedded_subprocess_escalation_missing_boundary",
        }
    );
}

#[test]
fn parser_top_level_escalation_end_requires_supported_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-top-level-escalation-end.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("top-level escalation end should reject root-only routing");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("escalation_end".to_string()).into(),
            detail: "escalation_end_requires_supported_parent_boundary",
        }
    );
}

#[test]
fn parser_top_level_escalation_throw_requires_supported_parent_boundary() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-top-level-escalation-throw.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("top-level escalation throw should reject root-only routing");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventConfiguration {
            process_id: ("main_process".to_string()).into(),
            node_id: ("escalation_throw".to_string()).into(),
            detail: "escalation_throw_requires_supported_parent_boundary",
        }
    );
}
