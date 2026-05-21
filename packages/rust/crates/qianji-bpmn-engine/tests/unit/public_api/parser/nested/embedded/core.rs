use super::{EMBEDDED_REVIEW_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package};

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
            process_id: ("main_process".to_string()).into(),
            node_id: ("inline_review".to_string()).into(),
            detail: "embedded_subprocess_start_event_count",
        }
    );
}
