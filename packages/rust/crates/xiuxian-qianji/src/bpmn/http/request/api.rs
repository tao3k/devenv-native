use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowCheckpointBackend,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusRequest,
    QianjiBpmnWorkflowTaskCompleteRequest,
};
use crate::bpmn::http_transport::error_api::QianjiBpmnWorkflowHttpError;
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
    pub process_id: String,
    /// Workflow instance identifier used for checkpoint lookup and fresh runs.
    pub instance_id: String,
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
            instance_id,
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
            instance_id,
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
        }
    }

    pub(in crate::bpmn::http_transport) fn into_task_complete_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowTaskCompleteRequest {
        QianjiBpmnWorkflowTaskCompleteRequest {
            bpmn_path: self.bpmn_path,
            dmn_paths: self.dmn_paths,
            instance_id,
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
        }
    }

    pub(in crate::bpmn::http_transport) fn into_cancel_request(
        self,
        instance_id: String,
    ) -> QianjiBpmnWorkflowCancelRequest {
        QianjiBpmnWorkflowCancelRequest {
            instance_id,
            checkpoint_backend: self.checkpoint_backend.into_control_backend(),
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
            instance_id,
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
