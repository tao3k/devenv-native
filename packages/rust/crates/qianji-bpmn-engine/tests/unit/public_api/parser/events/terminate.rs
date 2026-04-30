use super::parse_fixture_package;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEventKind, BpmnNodeKind};

#[test]
fn parser_terminate_end_materializes_event_binding() {
    let package = parse_fixture_package("terminate-end.bpmn", "terminate end should parse");
    let process = package
        .find_process("terminate_process")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::EndEvent);
    let event = process
        .event_for_node(1)
        .must("terminate end should materialize an event binding");
    assert_eq!(event.kind, BpmnEventKind::Terminate);
    assert_eq!(event.reference_id, None);
}
