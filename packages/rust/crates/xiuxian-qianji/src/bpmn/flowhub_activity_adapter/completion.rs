use serde_json::Value;
use xiuxian_qianji_control::{ControlResult, WorkerActivityTask};
use xiuxian_qianji_runtime::{
    build_flowhub_service_task_completion, flowhub_service_task_bpmn_source_path,
};

use crate::bpmn::control::{
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
};
use crate::bpmn::{
    QianjiBpmnWorkflowHttpCheckpointBackend, QianjiBpmnWorkflowTaskCompleteHttpRequest,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionHttpPayload,
};

/// Builds the BPMN service-task completion payload for a completed Flowhub
/// worker task.
///
/// # Errors
///
/// Returns a control error when the worker task lacks Flowhub service metadata,
/// the metadata schema is unsupported, required BPMN identity fields are
/// missing, the task is not a service task, or the supplied completion data is
/// missing required BPMN output fields.
pub fn build_flowhub_service_task_completion_payload(
    task: &WorkerActivityTask,
    data: Value,
) -> ControlResult<QianjiBpmnWorkflowTaskCompletionPayload> {
    let completion = build_flowhub_service_task_completion(task, data)?;
    Ok(QianjiBpmnWorkflowTaskCompletionPayload {
        token_id: completion.token_id.as_u64(),
        process_id: completion.process_id.into_string().into(),
        activity_id: completion.activity_id.into_string().into(),
        kind: QianjiBpmnWorkflowTaskCompletionKind::Service,
        data: completion.data,
        claimant: completion.claimant,
    })
}

/// Builds the BPMN task-completion HTTP request for a completed Flowhub worker
/// task.
///
/// # Errors
///
/// Returns a control error when the worker task lacks valid Flowhub service
/// metadata, the BPMN source path is missing, or the supplied completion data
/// fails required-output validation.
pub fn build_flowhub_service_task_complete_http_request(
    task: &WorkerActivityTask,
    data: Value,
) -> ControlResult<QianjiBpmnWorkflowTaskCompleteHttpRequest> {
    let bpmn_path = flowhub_service_task_bpmn_source_path(task)?;
    let completion = build_flowhub_service_task_completion_payload(task, data)?;

    Ok(QianjiBpmnWorkflowTaskCompleteHttpRequest {
        bpmn_path,
        dmn_paths: Vec::new(),
        checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend::RuntimeValkey,
        completion: QianjiBpmnWorkflowTaskCompletionHttpPayload {
            token_id: completion.token_id,
            process_id: completion.process_id,
            activity_id: completion.activity_id,
            kind: match completion.kind {
                QianjiBpmnWorkflowTaskCompletionKind::Send => {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::Send
                }
                QianjiBpmnWorkflowTaskCompletionKind::Service => {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::Service
                }
                QianjiBpmnWorkflowTaskCompletionKind::Script => {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::Script
                }
                QianjiBpmnWorkflowTaskCompletionKind::User => {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::User
                }
                QianjiBpmnWorkflowTaskCompletionKind::Manual => {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::Manual
                }
            },
            data: completion.data,
            claimant: completion.claimant,
        },
    })
}
