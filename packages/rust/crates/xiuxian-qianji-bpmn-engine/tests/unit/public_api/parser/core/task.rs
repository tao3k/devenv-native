use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{
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
fn parser_user_task_preserves_native_io_interaction_form_metadata() {
    let package = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "user-task-interaction.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  id="pkg_user_task_interaction">
  <bpmn:process id="user_task_interaction" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="ask_question">
      <bpmn:documentation>Review the generated question.</bpmn:documentation>
      <bpmn:ioSpecification>
        <bpmn:dataInput id="ask_question_interaction_type" name="interactionType"/>
        <bpmn:dataInput id="ask_question_question" name="question"/>
        <bpmn:dataInput id="ask_question_choices" name="choices"/>
        <bpmn:dataInput id="ask_question_free_text" name="freeText"/>
        <bpmn:dataOutput id="ask_question_answer" name="answer"/>
        <bpmn:inputSet>
          <bpmn:dataInputRefs>ask_question_interaction_type</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>ask_question_question</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>ask_question_choices</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>ask_question_free_text</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet>
          <bpmn:dataOutputRefs>ask_question_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:targetRef>ask_question_interaction_type</bpmn:targetRef>
        <bpmn:assignment>
          <bpmn:from>choice_input</bpmn:from>
          <bpmn:to>ask_question_interaction_type</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>currentQuestion</bpmn:sourceRef>
        <bpmn:targetRef>ask_question_question</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:targetRef>ask_question_choices</bpmn:targetRef>
        <bpmn:assignment>
          <bpmn:from>[{"value":"approve","label":"Approve"}]</bpmn:from>
          <bpmn:to>ask_question_choices</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:targetRef>ask_question_free_text</bpmn:targetRef>
        <bpmn:assignment>
          <bpmn:from>{"name":"feedback","optional":true}</bpmn:from>
          <bpmn:to>ask_question_free_text</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>ask_question_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="ask_question" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="ask_question" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must("user task interaction metadata should parse");
    let process = package
        .find_process("user_task_interaction")
        .must("process should be present");
    let form = process.nodes[1]
        .human_task_form
        .as_ref()
        .must("user task should preserve human task form metadata");

    assert_eq!(form.interaction_type.as_ref(), "choice_input");
    assert_eq!(form.question_ref.as_deref(), Some("currentQuestion"));
    assert_eq!(form.question_text.as_deref(), None);
    assert_eq!(form.choices_ref.as_deref(), None);
    assert_eq!(form.choices[0].value.as_ref(), "approve");
    assert_eq!(form.choices[0].label.as_deref(), Some("Approve"));
    assert_eq!(form.free_text_fields[0].name.as_ref(), "feedback");
    assert!(form.free_text_fields[0].optional);
    assert_eq!(form.result_output.as_deref(), Some("answer"));
}

#[test]
fn parser_user_task_preserves_standard_human_task_assignment_metadata() {
    let package = parse_bpmn_package(
        &[BpmnSourceFile::new(
            "user-task-assignment.bpmn",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL"
  id="pkg_user_task_assignment">
  <bpmn:process id="user_task_assignment" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review">
      <bpmn:humanPerformer name="reviewer">
        <bpmn:resourceAssignmentExpression>
          <bpmn:formalExpression>users.alice</bpmn:formalExpression>
        </bpmn:resourceAssignmentExpression>
      </bpmn:humanPerformer>
      <bpmn:potentialOwner name="review_team">
        <bpmn:resourceRef>reviewers</bpmn:resourceRef>
      </bpmn:potentialOwner>
    </bpmn:userTask>
    <bpmn:endEvent id="done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="review" />
    <bpmn:sequenceFlow id="flow_done" sourceRef="review" targetRef="done" />
  </bpmn:process>
</bpmn:definitions>"#,
        )],
        &BpmnParseOptions::default(),
    )
    .must("standard human-task assignment metadata should parse");
    let process = package
        .find_process("user_task_assignment")
        .must("process should be present");
    let assignment = process.nodes[1]
        .human_task_assignment
        .as_ref()
        .must("user task should preserve assignment metadata");

    assert_eq!(assignment.human_performers.len(), 1);
    assert_eq!(
        assignment.human_performers[0].name.as_deref(),
        Some("reviewer")
    );
    assert_eq!(
        assignment.human_performers[0]
            .assignment_expression
            .as_deref(),
        Some("users.alice")
    );
    assert_eq!(assignment.potential_owners.len(), 1);
    assert_eq!(
        assignment.potential_owners[0].name.as_deref(),
        Some("review_team")
    );
    assert_eq!(
        assignment.potential_owners[0].resource_ref.as_deref(),
        Some("reviewers")
    );
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
            process_id: ("missing_task_route".to_string()).into(),
            node_id: ("prepare_next".to_string()).into(),
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
            process_id: ("send_invoice_invalid".to_string()).into(),
            node_id: ("send_invoice_message".to_string()).into(),
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
