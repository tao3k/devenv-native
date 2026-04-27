pub(super) use super::super::runtime::{ok_of, write_wait_bundle};
pub(super) use super::super::valkey_support::TestValkey;
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
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload,
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
    <bpmn:userTask id="review_task" />
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
    <bpmn:userTask id="first_user" />
    <bpmn:serviceTask id="store_answer" />
    <bpmn:userTask id="second_user" />
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
