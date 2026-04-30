use super::{EMBEDDED_REVIEW_PROCESS_ID, fixture_source};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnNodeKind, BpmnParseOptions, parse_bpmn_package};

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
