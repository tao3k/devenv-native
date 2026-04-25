pub(super) use super::super::runtime::{ok_of, write_wait_bundle};
pub(super) use super::super::valkey_support::TestValkey;
pub(super) use crate::runtime_config::QianjiRuntimeEnv;
pub(super) use crate::{
    QianjiBpmnHostBridge, QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowInstancesRequest, QianjiBpmnWorkflowStartRequest, SchedulerAgentIdentity,
};
#[cfg(feature = "duckdb")]
pub(super) use crate::{
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowTaskCompleteRequest,
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
pub(super) fn write_service_task_bundle(temp_dir: &TempDir) -> PathBuf {
    let bpmn_path = temp_dir.path().join("service-task.bpmn");
    write_file(
        &bpmn_path,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_review">
  <bpmn:process id="review" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="review_task" />
    <bpmn:endEvent id="end" />
    <bpmn:sequenceFlow id="flow_1" sourceRef="start" targetRef="review_task" />
    <bpmn:sequenceFlow id="flow_2" sourceRef="review_task" targetRef="end" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    bpmn_path
}

fn write_file(path: &Path, contents: &str) {
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("should write bundle file {}: {error}", path.display()));
}
