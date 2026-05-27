use serde_json::Value;
use sha2::{Digest, Sha256};
use xiuxian_qianji_control::{ActivityResult, ControlError, ControlResult};

use crate::flowhub::{
    QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
};

/// Metadata key used on generic BPMN host-work completion results.
pub const BPMN_HOST_WORK_COMPLETION_METADATA_KEY: &str = "qianji_bpmn_host_work_completion";
/// Metadata schema used on generic BPMN host-work completion results.
pub const BPMN_HOST_WORK_COMPLETION_SCHEMA: &str = "xiuxian_qianji.bpmn.host_work_completion.v1";

/// Runtime-neutral BPMN host-work completion kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpmnHostWorkCompletionKind {
    /// Completion for BPMN generic task work.
    Task,
    /// Completion for BPMN send work.
    Send,
    /// Completion for BPMN service work.
    Service,
    /// Completion for BPMN script work.
    Script,
    /// Completion for BPMN user work.
    User,
    /// Completion for BPMN manual work.
    Manual,
}

/// Runtime-neutral BPMN host-work completion facts.
#[derive(Debug, Clone, PartialEq)]
pub struct BpmnHostWorkCompletion {
    /// Runtime token identifier for the pending host work.
    pub token_id: QianjiRuntimeBpmnTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiRuntimeBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiRuntimeBpmnActivityId,
    /// Completion kind supplied by the host.
    pub kind: BpmnHostWorkCompletionKind,
    /// Host-supplied payload merged into workflow variables.
    pub data: Value,
    /// Optional claimant supplied by the host when completing claimed work.
    pub claimant: Option<String>,
}

/// Builds a durable activity result from runtime-neutral BPMN host-work
/// completion facts.
///
/// # Errors
///
/// Returns a control error when completion data cannot be encoded for a stable
/// content hash.
pub fn build_bpmn_host_work_activity_result(
    completion: &BpmnHostWorkCompletion,
) -> ControlResult<ActivityResult> {
    let data_bytes = serde_json::to_vec(&completion.data).map_err(|error| ControlError::Codec {
        operation: "encode_bpmn_host_work_completion_data",
        message: error.to_string(),
    })?;
    Ok(ActivityResult {
        output_ref: None,
        output_hash: Some(sha256_digest(&data_bytes)),
        metadata: serde_json::json!({
            BPMN_HOST_WORK_COMPLETION_METADATA_KEY: {
                "schema": BPMN_HOST_WORK_COMPLETION_SCHEMA,
                "tokenId": completion.token_id.as_u64(),
                "processId": completion.process_id.as_str(),
                "activityId": completion.activity_id.as_str(),
                "kind": completion_kind_name(completion.kind),
                "data": completion.data.clone(),
                "claimant": completion.claimant
            }
        }),
    })
}

fn sha256_digest(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let digest = hasher.finalize();
    format!("sha256:{digest:x}")
}

fn completion_kind_name(kind: BpmnHostWorkCompletionKind) -> &'static str {
    match kind {
        BpmnHostWorkCompletionKind::Task => "task",
        BpmnHostWorkCompletionKind::Send => "send",
        BpmnHostWorkCompletionKind::Service => "service",
        BpmnHostWorkCompletionKind::Script => "script",
        BpmnHostWorkCompletionKind::User => "user",
        BpmnHostWorkCompletionKind::Manual => "manual",
    }
}
