pub(super) use super::{TestValkey, ok_of, unique_instance_id, write_wait_bundle};
pub(super) use crate::runtime_config::QianjiRuntimeEnv;
pub(super) use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowInterruptRequest,
    QianjiBpmnWorkflowStartRequest, SchedulerAgentIdentity,
};
#[cfg(feature = "duckdb")]
pub(super) use crate::{
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowTaskClaimPayload, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseRequest, QianjiBpmnWorkflowWorklistRequest,
    QianjiBpmnWorkflowWorklistRoutingFilter,
};
pub(super) use qianji_bpmn_engine::BpmnAdvanceOutcome;
#[cfg(feature = "duckdb")]
pub(super) use qianji_bpmn_engine::EventPollOutcome;
pub(super) use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
pub(super) use tempfile::TempDir;

pub(super) fn write_linear_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("linear.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_linear">
  <bpmn:process id="linear" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "duckdb")]
pub(super) fn write_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("user-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review_task">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_task_output_answer" name="answer" />
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "duckdb")]
pub(super) fn write_assignment_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("assignment-user-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review_assignment">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review_task">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_task_output_answer" name="answer" />
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
      <bpmn:humanPerformer name="reviewer">
        <bpmn:resourceAssignmentExpression>
          <bpmn:formalExpression>users.alice</bpmn:formalExpression>
        </bpmn:resourceAssignmentExpression>
      </bpmn:humanPerformer>
      <bpmn:potentialOwner name="review_team">
        <bpmn:resourceRef>reviewers</bpmn:resourceRef>
      </bpmn:potentialOwner>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "duckdb")]
pub(super) fn write_lane_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("lane-user-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review_lane">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:laneSet id="LaneSet_Review" name="Ownership">
      <bpmn:lane id="Lane_Reviewer" name="Reviewer Lane">
        <bpmn:flowNodeRef>review_task</bpmn:flowNodeRef>
      </bpmn:lane>
    </bpmn:laneSet>
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review_task">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="review_task_output_answer" name="answer" />
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
      <bpmn:humanPerformer name="reviewer">
        <bpmn:resourceAssignmentExpression>
          <bpmn:formalExpression>users.alice</bpmn:formalExpression>
        </bpmn:resourceAssignmentExpression>
      </bpmn:humanPerformer>
      <bpmn:potentialOwner name="review_team">
        <bpmn:resourceRef>reviewers</bpmn:resourceRef>
      </bpmn:potentialOwner>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "duckdb")]
pub(super) fn write_form_user_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("form-user-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review_form">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="review_task">
      <bpmn:documentation>Ask the current question.</bpmn:documentation>
      <bpmn:ioSpecification>
        <bpmn:dataInput id="review_task_input_interactionType" name="interactionType" />
        <bpmn:dataInput id="review_task_input_question" name="question" />
        <bpmn:dataInput id="review_task_input_choices" name="choices" />
        <bpmn:dataInput id="review_task_input_freeText" name="freeText" />
        <bpmn:dataOutput id="review_task_output_answer" name="answer" />
        <bpmn:inputSet id="review_task_input_set">
          <bpmn:dataInputRefs>review_task_input_interactionType</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>review_task_input_question</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>review_task_input_choices</bpmn:dataInputRefs>
          <bpmn:dataInputRefs>review_task_input_freeText</bpmn:dataInputRefs>
        </bpmn:inputSet>
        <bpmn:outputSet id="review_task_output_set">
          <bpmn:dataOutputRefs>review_task_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>choice_input</bpmn:from>
          <bpmn:to>review_task_input_interactionType</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>currentQuestion</bpmn:sourceRef>
        <bpmn:targetRef>review_task_input_question</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:sourceRef>currentChoices</bpmn:sourceRef>
        <bpmn:targetRef>review_task_input_choices</bpmn:targetRef>
      </bpmn:dataInputAssociation>
      <bpmn:dataInputAssociation>
        <bpmn:assignment>
          <bpmn:from>{"name":"feedback","optional":true}</bpmn:from>
          <bpmn:to>review_task_input_freeText</bpmn:to>
        </bpmn:assignment>
      </bpmn:dataInputAssociation>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>review_task_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>answer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

#[cfg(feature = "duckdb")]
pub(super) fn write_user_service_user_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("user-service-user.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_user_service_user">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:userTask id="first_user">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="first_user_output_answer" name="answer" />
        <bpmn:outputSet id="first_user_output_set">
          <bpmn:dataOutputRefs>first_user_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>first_user_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>firstAnswer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:serviceTask id="store_answer">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="store_answer_output_stored" name="stored" />
        <bpmn:outputSet id="store_answer_output_set">
          <bpmn:dataOutputRefs>store_answer_output_stored</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>store_answer_output_stored</bpmn:sourceRef>
        <bpmn:targetRef>stored</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:serviceTask>
    <bpmn:userTask id="second_user">
      <bpmn:ioSpecification>
        <bpmn:dataOutput id="second_user_output_answer" name="answer" />
        <bpmn:outputSet id="second_user_output_set">
          <bpmn:dataOutputRefs>second_user_output_answer</bpmn:dataOutputRefs>
        </bpmn:outputSet>
      </bpmn:ioSpecification>
      <bpmn:dataOutputAssociation>
        <bpmn:sourceRef>second_user_output_answer</bpmn:sourceRef>
        <bpmn:targetRef>secondAnswer</bpmn:targetRef>
      </bpmn:dataOutputAssociation>
    </bpmn:userTask>
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="first_user" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="first_user" targetRef="store_answer" />
    <bpmn:sequenceFlow id="flow_3" sourceRef="store_answer" targetRef="second_user" />
    <bpmn:sequenceFlow id="flow_4" sourceRef="second_user" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("should write bundle file {}: {error}", path.display()));
}
