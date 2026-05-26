use std::path::{Path, PathBuf};

use super::support::{QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowEventPollRequest};

#[test]
fn event_poll_request_derives_resume_request() {
    let request = QianjiBpmnWorkflowEventPollRequest {
        bpmn_path: PathBuf::from("flows/agent-coding.bpmn"),
        dmn_paths: vec![PathBuf::from("flows/rules.dmn")],
        instance_id: "workflow-event-poll".to_string().into(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
    };

    let resume_request = request.workflow_resume_request();

    assert_eq!(
        resume_request.bpmn_path.as_path(),
        Path::new("flows/agent-coding.bpmn")
    );
    assert_eq!(
        resume_request.dmn_paths.as_slice(),
        [PathBuf::from("flows/rules.dmn")].as_slice()
    );
    assert_eq!(resume_request.instance_id.as_ref(), "workflow-event-poll");
    assert_eq!(
        resume_request.checkpoint_backend,
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb
    );
}
