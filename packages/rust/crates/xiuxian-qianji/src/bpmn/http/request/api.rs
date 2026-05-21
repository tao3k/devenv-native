//! HTTP request DTOs and control-request adapters for BPMN workflow routes.

use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskClaimPayload, QianjiBpmnWorkflowTaskClaimRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseRequest,
};
use crate::bpmn::http_transport::error_api::QianjiBpmnWorkflowHttpError;
use crate::bpmn::identity::{
    QianjiBpmnActivityId, QianjiBpmnProcessId, QianjiBpmnWorkflowInstanceId,
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
    fn into_control_backend(self) -> QianjiBpmnWorkflowCheckpointBackend {
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
            start_at_node_id: None,
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

/// JSON host-work result kind accepted by explicit task completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiBpmnWorkflowTaskCompletionHttpKind {
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
    /// User- or operator-supplied payload merged into workflow variables.
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
                kind: match self.completion.kind {
                    QianjiBpmnWorkflowTaskCompletionHttpKind::User => {
                        QianjiBpmnWorkflowTaskCompletionKind::User
                    }
                    QianjiBpmnWorkflowTaskCompletionHttpKind::Manual => {
                        QianjiBpmnWorkflowTaskCompletionKind::Manual
                    }
                },
                data: self.completion.data,
                claimant: self.completion.claimant,
            },
            continue_until_human_boundary: false,
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
