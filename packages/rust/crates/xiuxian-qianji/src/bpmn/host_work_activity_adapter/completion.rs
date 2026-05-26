//! BPMN host-work completion adapters for durable activity evidence.

use xiuxian_qianji_control::{ActivityResult, ControlResult};
use xiuxian_qianji_runtime::{
    BpmnHostWorkCompletion, BpmnHostWorkCompletionKind, QianjiRuntimeBpmnActivityId,
    QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
    build_bpmn_host_work_activity_result as build_runtime_bpmn_host_work_activity_result,
};

use crate::bpmn::control::{
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
};

/// Builds a durable activity result from a qianji-server BPMN host-work
/// completion payload.
///
/// # Errors
///
/// Returns a control error when the runtime completion data cannot be encoded
/// for a stable content hash.
pub fn build_bpmn_host_work_activity_result(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> ControlResult<ActivityResult> {
    build_runtime_bpmn_host_work_activity_result(&bpmn_host_work_completion_from_payload(
        completion,
    ))
}

pub(crate) fn bpmn_host_work_completion_from_payload(
    completion: &QianjiBpmnWorkflowTaskCompletionPayload,
) -> BpmnHostWorkCompletion {
    BpmnHostWorkCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(completion.token_id),
        process_id: QianjiRuntimeBpmnProcessId::new(completion.process_id.as_str()),
        activity_id: QianjiRuntimeBpmnActivityId::new(completion.activity_id.as_str()),
        kind: runtime_completion_kind(completion.kind),
        data: completion.data.clone(),
        claimant: completion.claimant.clone(),
    }
}

fn runtime_completion_kind(
    kind: QianjiBpmnWorkflowTaskCompletionKind,
) -> BpmnHostWorkCompletionKind {
    match kind {
        QianjiBpmnWorkflowTaskCompletionKind::Send => BpmnHostWorkCompletionKind::Send,
        QianjiBpmnWorkflowTaskCompletionKind::Service => BpmnHostWorkCompletionKind::Service,
        QianjiBpmnWorkflowTaskCompletionKind::Script => BpmnHostWorkCompletionKind::Script,
        QianjiBpmnWorkflowTaskCompletionKind::User => BpmnHostWorkCompletionKind::User,
        QianjiBpmnWorkflowTaskCompletionKind::Manual => BpmnHostWorkCompletionKind::Manual,
    }
}
