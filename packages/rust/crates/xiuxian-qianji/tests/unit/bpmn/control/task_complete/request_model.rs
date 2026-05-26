use std::path::{Path, PathBuf};

use super::{
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowTaskCompleteBatchRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, json,
};

#[test]
fn task_complete_request_derives_resume_request() {
    let request = QianjiBpmnWorkflowTaskCompleteRequest {
        bpmn_path: PathBuf::from("flows/agent-coding.bpmn"),
        dmn_paths: vec![PathBuf::from("flows/rules.dmn")],
        instance_id: "workflow-complete".to_string().into(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        completion: completion_payload(7),
        continue_until_human_boundary: false,
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
    assert_eq!(resume_request.instance_id.as_ref(), "workflow-complete");
    assert_eq!(
        resume_request.checkpoint_backend,
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb
    );
}

#[test]
fn task_complete_batch_request_derives_resume_request() {
    let request = QianjiBpmnWorkflowTaskCompleteBatchRequest {
        bpmn_path: PathBuf::from("flows/agent-coding.bpmn"),
        dmn_paths: vec![PathBuf::from("flows/rules.dmn")],
        instance_id: "workflow-batch".to_string().into(),
        checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb,
        completions: vec![completion_payload(7), completion_payload(8)],
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
    assert_eq!(resume_request.instance_id.as_ref(), "workflow-batch");
    assert_eq!(
        resume_request.checkpoint_backend,
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb
    );
}

fn completion_payload(token_id: u64) -> QianjiBpmnWorkflowTaskCompletionPayload {
    QianjiBpmnWorkflowTaskCompletionPayload {
        token_id,
        process_id: "agent_coding".into(),
        activity_id: "resolve_project".into(),
        kind: QianjiBpmnWorkflowTaskCompletionKind::Service,
        data: json!({"projectResolved": true}),
        claimant: None,
    }
}
