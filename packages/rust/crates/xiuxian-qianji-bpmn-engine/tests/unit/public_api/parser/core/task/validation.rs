use crate::public_api::parser::core::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnEngineError, BpmnNodeKind, BpmnParseOptions, BpmnSourceFile, parse_bpmn_package,
};

#[test]
fn parser_service_task_requires_single_outgoing_route() {
    let error = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "missing-task-route.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_missing_task_route">
  <bpmn:process id="missing_task_route" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="prepare_next" />
    <bpmn:exclusiveGateway id="more_questions" default="flow_done" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="prepare_next" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="more_questions" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must_err("service task without an outgoing route should fail validation");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: ("missing_task_route".to_string()).into(),
            node_id: ("prepare_next".to_string()).into(),
            detail: "task_requires_single_outgoing",
        }
    );
}

#[test]
fn parser_send_task_without_message_binding_materializes_metadata_only_node() {
    let package = parse_fixture_package("invalid-send-task-missing-message-binding.bpmn");
    let process = package
        .find_process("send_invoice_invalid")
        .must("process should be present");
    let node = &process.nodes[1];

    assert_eq!(node.kind, BpmnNodeKind::SendTask);
    assert!(process.event_for_node(1).is_none());
}

#[test]
fn parser_receive_task_rejects_multiple_message_binding_sources() {
    let error = parse_fixture_error(
        "invalid-receive-task-double-message-binding.bpmn",
        "receive task should reject multiple message binding sources",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: ("await_invoice_invalid".to_string()).into(),
            node_id: ("await_invoice_message".to_string()).into(),
            detail: "multiple_task_message_bindings",
        }
    );
}

#[test]
fn parser_receive_task_rejects_non_message_event_binding() {
    let error = parse_fixture_error(
        "invalid-receive-task-signal-binding.bpmn",
        "receive task should stay message-only in the bounded slice",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedTaskConfiguration {
            process_id: ("await_invoice_invalid_signal".to_string()).into(),
            node_id: ("await_invoice_message".to_string()).into(),
            detail: "unsupported_receive_task_event_kind",
        }
    );
}
