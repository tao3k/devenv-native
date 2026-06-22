use crate::public_api::parser::core::parse_fixture_package;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
    BpmnEventKind, BpmnNodeKind, BpmnParseOptions, BpmnSourceFile, parse_bpmn_package,
};

#[test]
fn parser_generic_task_materializes_native_task_kind() {
    let package = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "generic-task-basic.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  id="pkg_generic_task">
  <bpmn:process id="generic_task" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:task id="do_work" />
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="do_work" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="do_work" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must("generic BPMN task should parse");
    let process = package
        .find_process("generic_task")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Task);
}

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
