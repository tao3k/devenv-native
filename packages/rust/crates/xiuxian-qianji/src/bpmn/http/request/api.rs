//! HTTP request DTOs and control-request adapters for BPMN workflow routes.

use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimPayload, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteBatchRequest, QianjiBpmnWorkflowTaskCompleteRequest,
    QianjiBpmnWorkflowTaskCompletionKind, QianjiBpmnWorkflowTaskCompletionPayload,
    QianjiBpmnWorkflowTaskReleasePayload, QianjiBpmnWorkflowTaskReleaseRequest,
};
use crate::bpmn::http_transport::error_api::QianjiBpmnWorkflowHttpError;
use crate::bpmn::identity::{
    QianjiBpmnActivityId, QianjiBpmnProcessId, QianjiBpmnStartAtNodeId,
    QianjiBpmnWorkflowInstanceId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// JSON checkpoint backend selector for BPMN workflow HTTP requests.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QianjiBpmnWorkflowHttpCheckpointBackend {
    /// Resolve the runtime-configured Valkey checkpoint backend.
    #[default]
    RuntimeValkey,
}

impl QianjiBpmnWorkflowHttpCheckpointBackend {
    pub(in crate::bpmn::http_transport) fn into_control_backend(
        self,
    ) -> QianjiBpmnWorkflowCheckpointBackend {
        match self {
            Self::RuntimeValkey => QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
        }
    }
}

/// JSON body for starting one BPMN workflow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowStartHttpRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    #[serde(default)]
    pub dmn_paths: Vec<PathBuf>,
    /// BPMN process identifier used for a fresh run.
    pub process_id: QianjiBpmnProcessId,
    /// Workflow instance identifier used for checkpoint lookup and fresh runs.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Optional initial variables for a fresh run.
    #[serde(default)]
    pub initial_variables: Option<Value>,
    /// Optional BPMN node id for a fresh synthetic start-at run.
    #[serde(default)]
    pub start_at_node_id: Option<QianjiBpmnStartAtNodeId>,
    /// Optional checkpoint backend to use for this bounded run. HTTP service
    /// mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
}

impl QianjiBpmnWorkflowStartHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_control_request(
        self,
    ) -> QianjiBpmnWorkflowStartRequest {
        QianjiBpmnWorkflowStartRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            process_id: self.process_id,
            instance_id: self.instance_id,
            initial_variables: self.initial_variables,
            start_at_node_id: self.start_at_node_id,
            checkpoint_backend: Some(self.checkpoint_backend.into_control_backend()),
        }
    }
}

/// JSON body for checkpoint-backed BPMN workflow actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowActionHttpRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    #[serde(default)]
    pub dmn_paths: Vec<PathBuf>,
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
}

impl QianjiBpmnWorkflowActionHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_resume_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowResumeRequest {
        QianjiBpmnWorkflowResumeRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
        }
    }

    pub(in crate::bpmn::http_transport) fn into_event_poll_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowEventPollRequest {
        QianjiBpmnWorkflowEventPollRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
        }
    }

    pub(in crate::bpmn::http_transport) fn into_cancel_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowCancelRequest {
        QianjiBpmnWorkflowCancelRequest {
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
        }
    }
}

/// JSON body for explicitly applying a control-ledger recovery plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiControlRecoveryApplyHttpRequest {
    /// Event timestamp supplied by the caller.
    pub occurred_at_ms: u64,
    /// Recovery attempt number.
    pub attempt: u32,
    /// Human-readable reason for this recovery attempt.
    pub reason: String,
    /// Maximum attempts permitted by the recovery policy.
    pub max_attempts: u32,
    /// Backoff attached to retryable activity recovery.
    #[serde(default)]
    pub backoff_ms: u64,
    /// Whether the recovery policy requires human approval.
    #[serde(default)]
    pub require_human_approval: bool,
    /// Queue priority for applied retry work.
    #[serde(default)]
    pub priority: i64,
}

/// JSON host-work result kind accepted by explicit task completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiBpmnWorkflowTaskCompletionHttpKind {
    /// Complete a BPMN `sendTask`.
    Send,
    /// Complete a BPMN `serviceTask`.
    Service,
    /// Complete a BPMN `scriptTask`.
    Script,
    /// Complete a BPMN `userTask`.
    User,
    /// Complete a BPMN `manualTask`.
    Manual,
}

/// JSON payload for one explicit pending host-work completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskCompletionHttpPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiBpmnActivityId,
    /// Pending host-work result kind.
    pub kind: QianjiBpmnWorkflowTaskCompletionHttpKind,
    /// Worker-, user-, or operator-supplied payload merged into workflow
    /// variables.
    pub data: Value,
    /// Optional claimant supplied by the host when completing claimed human
    /// work.
    #[serde(default)]
    pub claimant: Option<String>,
}

/// JSON body for checkpoint-backed BPMN task completion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskCompleteHttpRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    #[serde(default)]
    pub dmn_paths: Vec<PathBuf>,
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
    /// Explicit completion payload for the pending host task.
    pub completion: QianjiBpmnWorkflowTaskCompletionHttpPayload,
}

impl QianjiBpmnWorkflowTaskCompleteHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_task_complete_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowTaskCompleteRequest {
        QianjiBpmnWorkflowTaskCompleteRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
            completion: QianjiBpmnWorkflowTaskCompletionPayload {
                token_id: self.completion.token_id,
                process_id: self.completion.process_id,
                activity_id: self.completion.activity_id,
                kind: http_completion_kind_into_control(self.completion.kind),
                data: self.completion.data,
                claimant: self.completion.claimant,
            },
            continue_until_human_boundary: false,
        }
    }
}

/// JSON body for checkpoint-backed BPMN task-completion batches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskCompleteBatchHttpRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    #[serde(default)]
    pub dmn_paths: Vec<PathBuf>,
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
    /// Explicit completion payloads for pending host tasks.
    pub completions: Vec<QianjiBpmnWorkflowTaskCompletionHttpPayload>,
}

impl QianjiBpmnWorkflowTaskCompleteBatchHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_task_complete_batch_request(
        self,
        instance_id: String,
    ) -> Result<QianjiBpmnWorkflowTaskCompleteBatchRequest, QianjiBpmnWorkflowHttpError> {
        if self.completions.is_empty() {
            return Err(QianjiBpmnWorkflowHttpError::bad_request(
                "empty_task_completion_batch",
                "completions must contain at least one task completion",
            ));
        }

        Ok(QianjiBpmnWorkflowTaskCompleteBatchRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            instance_id: QianjiBpmnWorkflowInstanceId::from(instance_id),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
            completions: self
                .completions
                .into_iter()
                .map(|completion| QianjiBpmnWorkflowTaskCompletionPayload {
                    token_id: completion.token_id,
                    process_id: completion.process_id,
                    activity_id: completion.activity_id,
                    kind: http_completion_kind_into_control(completion.kind),
                    data: completion.data,
                    claimant: completion.claimant,
                })
                .collect(),
        })
    }
}

/// JSON payload for recording one failed pending host-work attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskFailureHttpPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiBpmnActivityId,
    /// Pending host-work kind.
    pub kind: QianjiBpmnWorkflowTaskCompletionHttpKind,
    /// Stable failure code for the durable `ActivityTask` event.
    pub error_code: String,
    /// Human-readable failure message.
    pub message: String,
    /// Whether the failure may be retried by a later recovery slice.
    #[serde(default)]
    pub retryable: bool,
    /// Optional caller-supplied audit metadata.
    #[serde(default)]
    pub metadata: Value,
}

/// JSON body for checkpoint-backed BPMN task failure evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskFailHttpRequest {
    /// Filesystem path to the BPMN source.
    pub bpmn_path: PathBuf,
    /// Optional DMN sources loaded alongside the BPMN package.
    #[serde(default)]
    pub dmn_paths: Vec<PathBuf>,
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
    /// Explicit failure payload for the pending host task.
    pub failure: QianjiBpmnWorkflowTaskFailureHttpPayload,
}

impl QianjiBpmnWorkflowTaskFailHttpRequest {
    pub(in crate::bpmn::http_transport) fn workflow_resume_request(
        &self,
        instance_id: QianjiBpmnWorkflowInstanceId,
    ) -> QianjiBpmnWorkflowResumeRequest {
        QianjiBpmnWorkflowResumeRequest {
            bpmn_path: self.bpmn_path.clone(),
            dmn_paths: self.dmn_paths.clone(),
            instance_id,
            checkpoint_backend: self.checkpoint_backend.clone().into_control_backend(),
        }
    }
}

fn http_completion_kind_into_control(
    kind: QianjiBpmnWorkflowTaskCompletionHttpKind,
) -> QianjiBpmnWorkflowTaskCompletionKind {
    match kind {
        QianjiBpmnWorkflowTaskCompletionHttpKind::Send => {
            QianjiBpmnWorkflowTaskCompletionKind::Send
        }
        QianjiBpmnWorkflowTaskCompletionHttpKind::Service => {
            QianjiBpmnWorkflowTaskCompletionKind::Service
        }
        QianjiBpmnWorkflowTaskCompletionHttpKind::Script => {
            QianjiBpmnWorkflowTaskCompletionKind::Script
        }
        QianjiBpmnWorkflowTaskCompletionHttpKind::User => {
            QianjiBpmnWorkflowTaskCompletionKind::User
        }
        QianjiBpmnWorkflowTaskCompletionHttpKind::Manual => {
            QianjiBpmnWorkflowTaskCompletionKind::Manual
        }
    }
}

/// JSON payload for one explicit pending human-task claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskClaimHttpPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiBpmnActivityId,
    /// Host- or operator-facing claimant identifier.
    pub claimant: String,
}

/// JSON body for checkpoint-backed BPMN human-task claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskClaimHttpRequest {
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
    /// Explicit claim payload for the pending human task.
    pub claim: QianjiBpmnWorkflowTaskClaimHttpPayload,
}

impl QianjiBpmnWorkflowTaskClaimHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_task_claim_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowTaskClaimRequest {
        QianjiBpmnWorkflowTaskClaimRequest {
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
            claim: QianjiBpmnWorkflowTaskClaimPayload {
                token_id: self.claim.token_id,
                process_id: self.claim.process_id,
                activity_id: self.claim.activity_id,
                claimant: self.claim.claimant,
            },
        }
    }
}

/// JSON payload for one explicit pending human-task claim release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskReleaseHttpPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiBpmnActivityId,
    /// Host- or operator-facing claimant identifier that currently owns the
    /// work.
    pub claimant: String,
}

/// JSON body for checkpoint-backed BPMN human-task claim release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskReleaseHttpRequest {
    /// Checkpoint backend that already owns persisted workflow state. HTTP
    /// service mode defaults to runtime-configured Valkey when omitted.
    #[serde(default)]
    pub checkpoint_backend: QianjiBpmnWorkflowHttpCheckpointBackend,
    /// Explicit release payload for the pending human task.
    pub release: QianjiBpmnWorkflowTaskReleaseHttpPayload,
}

impl QianjiBpmnWorkflowTaskReleaseHttpRequest {
    pub(in crate::bpmn::http_transport) fn into_task_release_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowTaskReleaseRequest {
        QianjiBpmnWorkflowTaskReleaseRequest {
            instance_id: instance_id.into(),
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
            release: QianjiBpmnWorkflowTaskReleasePayload {
                token_id: self.release.token_id,
                process_id: self.release.process_id,
                activity_id: self.release.activity_id,
                claimant: self.release.claimant,
            },
        }
    }
}

/// Query parameters for loading checkpoint-backed BPMN workflow status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowStatusHttpQuery {
    /// Checkpoint backend kind. HTTP service mode accepts only
    /// `runtime_valkey` and defaults to it when omitted.
    #[serde(default)]
    pub checkpoint_backend: Option<String>,
}

impl QianjiBpmnWorkflowStatusHttpQuery {
    pub(in crate::bpmn::http_transport) fn into_status_request(
        self,
        instance_id: String,
    ) -> Result<QianjiBpmnWorkflowStatusRequest, QianjiBpmnWorkflowHttpError> {
        Ok(QianjiBpmnWorkflowStatusRequest {
            instance_id: instance_id.into(),
            checkpoint_backend: self.into_control_backend()?,
        })
    }

    fn into_control_backend(
        self,
    ) -> Result<QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowHttpError> {
        let Some(checkpoint_backend) = self.checkpoint_backend else {
            return Ok(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey);
        };
        match checkpoint_backend.as_str() {
            "runtime_valkey" | "valkey" => Ok(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
            _ => Err(QianjiBpmnWorkflowHttpError::bad_request(
                "unsupported_checkpoint_backend",
                "checkpoint_backend must be `runtime_valkey`",
            )),
        }
    }
}
