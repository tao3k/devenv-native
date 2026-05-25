use super::{EMBEDDED_REVIEW_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package,
};

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
            process_id: ("main_process".to_string()).into(),
            node_id: ("inline_review".to_string()).into(),
            detail: "embedded_subprocess_error_missing_boundary",
        }
    );
}
