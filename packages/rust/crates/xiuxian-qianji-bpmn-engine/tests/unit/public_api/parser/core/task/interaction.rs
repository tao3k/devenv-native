use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnParseOptions, BpmnSourceFile, parse_bpmn_package};

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
