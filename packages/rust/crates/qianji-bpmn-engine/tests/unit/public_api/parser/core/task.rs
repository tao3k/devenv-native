use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    BpmnEngineError, BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSourceFile,
    parse_bpmn_package,
};

#[test]
fn parser_send_task_message_ref_materializes_message_event_binding() {
    let package = parse_fixture_package("send-task-basic.bpmn");
    let process = package
        .find_process("send_invoice")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::SendTask);
    let event = process
        .event_for_node(1)
        .must("send task should materialize a message binding");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("invoice_dispatched"));
    assert_eq!(event.name.as_deref(), Some("send_invoice_message"));
}

#[test]
fn parser_receive_task_nested_message_event_materializes_message_binding() {
    let package = parse_fixture_package("receive-task-basic.bpmn");
    let process = package
        .find_process("await_invoice")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::ReceiveTask);
    let event = process
        .event_for_node(1)
        .must("receive task should materialize a message binding");
    assert_eq!(event.kind, BpmnEventKind::Message);
    assert_eq!(event.reference_id.as_deref(), Some("invoice_received"));
    assert_eq!(event.name.as_deref(), Some("await_invoice_message"));
}

#[test]
fn parser_script_task_preserves_bounded_script_metadata() {
    let package = parse_fixture_package("script-task-basic.bpmn");
    let process = package
        .find_process("evaluate_script")
        .must("process should be present");
    let task = &process.nodes[1];

    assert_eq!(task.kind, BpmnNodeKind::ScriptTask);
    let script = task
        .script_task
        .as_ref()
        .must("script task metadata should be preserved");
    assert_eq!(script.script_format.as_deref(), Some("feel"));
    assert_eq!(script.script_body.as_deref(), Some("result = amount + tax"));
}

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
            process_id: "missing_task_route".to_string(),
            node_id: "prepare_next".to_string(),
            detail: "task_requires_single_outgoing",
        }
    );
}

#[test]
fn parser_send_task_requires_one_message_binding() {
    let error = parse_fixture_error(
        "invalid-send-task-missing-message-binding.bpmn",
        "send task without a message binding should fail validation",
    );

    assert_eq!(
        error,
        BpmnEngineError::MissingRequiredNodeElement {
            process_id: "send_invoice_invalid".to_string(),
            node_id: "send_invoice_message".to_string(),
            element: "message_binding",
        }
    );
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
            process_id: "await_invoice_invalid".to_string(),
            node_id: "await_invoice_message".to_string(),
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
            process_id: "await_invoice_invalid_signal".to_string(),
            node_id: "await_invoice_message".to_string(),
            detail: "unsupported_receive_task_event_kind",
        }
    );
}
