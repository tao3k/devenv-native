use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnNodeKind, BpmnParseOptions, parse_bpmn_package};

#[test]
fn parser_call_activity_materializes_called_process_reference() {
    let package = parse_bpmn_package(
        &[fixture_source("call-activity-basic.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("bounded call activity should parse");
    let process = package
        .find_process("main_process")
        .must("main process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SubProcess);
    assert_eq!(
        process.nodes[1].called_process_id.as_deref(),
        Some("child_process")
    );
    assert!(package.find_process("child_process").is_some());
}

#[test]
fn parser_call_activity_missing_target_is_rejected() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-call-activity-missing-target.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("call activity should target an existing process");

    assert_eq!(
        error,
        BpmnEngineError::UnknownCalledProcess {
            process_id: "main_process".to_string(),
            node_id: "invoke_child".to_string(),
            called_process_id: "missing_process".to_string(),
        }
    );
}
