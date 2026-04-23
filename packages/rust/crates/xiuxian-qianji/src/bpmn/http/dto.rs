use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowCancelRequest,
    QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowEventPollRequest, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStartRequest, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowTaskCompleteRequest,
};
use crate::bpmn::driver::QianjiBpmnExecutionReport;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use qianji_bpmn_engine::{BpmnAdvanceOutcome, BpmnInstanceState, InstanceLifecycle};
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
    /// Use one lightweight local `SQLite` checkpoint database.
    #[cfg(feature = "sqlite")]
    Sqlite {
        /// Filesystem path to the `SQLite` checkpoint database.
        path: PathBuf,
    },
}

impl QianjiBpmnWorkflowHttpCheckpointBackend {
    pub(super) fn into_control_backend(self) -> QianjiBpmnWorkflowCheckpointBackend {
        match self {
            Self::RuntimeValkey => QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey,
            #[cfg(feature = "sqlite")]
            Self::Sqlite { path } => QianjiBpmnWorkflowCheckpointBackend::Sqlite(path),
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
    pub(super) fn into_control_request(self) -> QianjiBpmnWorkflowStartRequest {
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
    pub(super) fn into_resume_request(
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

    pub(super) fn into_event_poll_request(
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

    pub(super) fn into_task_complete_request(
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

    pub(super) fn into_cancel_request(
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
    /// Checkpoint backend kind: `runtime_valkey` or `sqlite`. HTTP service
    /// mode defaults to `runtime_valkey` when omitted.
    #[serde(default)]
    pub checkpoint_backend: Option<String>,
    /// Filesystem path to the `SQLite` checkpoint database when
    /// `checkpoint_backend=sqlite`.
    #[serde(default)]
    pub sqlite_path: Option<PathBuf>,
}

impl QianjiBpmnWorkflowStatusHttpQuery {
    pub(super) fn into_control_backend(
        self,
    ) -> Result<QianjiBpmnWorkflowCheckpointBackend, QianjiBpmnWorkflowHttpError> {
        let Some(checkpoint_backend) = self.checkpoint_backend else {
            return Ok(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey);
        };
        match checkpoint_backend.as_str() {
            "runtime_valkey" | "valkey" => Ok(QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey),
            #[cfg(feature = "sqlite")]
            "sqlite" => self
                .sqlite_path
                .map(QianjiBpmnWorkflowCheckpointBackend::Sqlite)
                .ok_or_else(|| {
                    QianjiBpmnWorkflowHttpError::bad_request(
                        "missing_sqlite_path",
                        "`sqlite_path` is required when checkpoint_backend=sqlite",
                    )
                }),
            _ => Err(QianjiBpmnWorkflowHttpError::bad_request(
                "unsupported_checkpoint_backend",
                "checkpoint_backend must be `runtime_valkey` or `sqlite`",
            )),
        }
    }
}

/// Compact runtime snapshot embedded in HTTP responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowSnapshotHttpResponse {
    /// Stable workflow instance identifier.
    pub instance_id: String,
    /// Stable BPMN process identifier.
    pub process_id: String,
    /// Monotonic runtime sequence.
    pub sequence: u64,
    /// High-level BPMN instance lifecycle.
    pub lifecycle: InstanceLifecycle,
    /// Current workflow variables.
    pub variables: Value,
    /// Number of active host-work items.
    pub pending_host_work_count: usize,
    /// Number of active external wait registrations.
    pub wait_registration_count: usize,
    /// Number of active runtime tokens.
    pub active_token_count: usize,
}

impl QianjiBpmnWorkflowSnapshotHttpResponse {
    fn from_instance(instance: &BpmnInstanceState) -> Self {
        Self {
            instance_id: instance.instance_id.to_string(),
            process_id: instance.process.process_id.to_string(),
            sequence: instance.sequence,
            lifecycle: instance.lifecycle.clone(),
            variables: instance.variables.clone(),
            pending_host_work_count: instance.pending_host_work.len(),
            wait_registration_count: instance.waits.len(),
            active_token_count: instance.active_tokens.len(),
        }
    }
}

/// HTTP response for one BPMN workflow execution action.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowRunHttpResponse {
    /// Stable engine outcome emitted by the execution facade.
    pub outcome: BpmnAdvanceOutcome,
    /// Whether the run resumed from a stored checkpoint.
    pub resumed_from_checkpoint: bool,
    /// Whether the driver saved a checkpoint after the run.
    pub checkpoint_saved: bool,
    /// Whether the driver deleted stored checkpoint state after a terminal run.
    pub checkpoint_deleted: bool,
    /// Checkpoint backend used by the action, if any.
    pub checkpoint_backend: Option<String>,
    /// Runtime snapshot after the action.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowRunHttpResponse {
    pub(super) fn from_start_report(report: &QianjiBpmnWorkflowStartReport) -> Self {
        Self::from_execution_report(&report.execution, report.checkpoint_store.as_ref())
    }

    fn from_execution_report(
        execution: &QianjiBpmnExecutionReport,
        checkpoint_store: Option<&crate::bpmn::backend::QianjiBpmnCheckpointStore>,
    ) -> Self {
        Self {
            outcome: execution.outcome.clone(),
            resumed_from_checkpoint: execution.resumed_from_checkpoint,
            checkpoint_saved: execution.checkpoint_saved,
            checkpoint_deleted: execution.checkpoint_deleted,
            checkpoint_backend: checkpoint_store.map(|store| store.backend_name().to_string()),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(
                execution.session.instance(),
            ),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN workflow status load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowStatusHttpResponse {
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Runtime snapshot loaded from storage.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowStatusHttpResponse {
    pub(super) fn from_report(report: &QianjiBpmnWorkflowStatusReport) -> Self {
        Self {
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN workflow cancellation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowCancelHttpResponse {
    /// Whether a checkpoint was deleted.
    pub cancelled: bool,
    /// Monotonic checkpoint sequence loaded before deletion.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Runtime snapshot loaded before deletion.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowCancelHttpResponse {
    pub(super) fn from_report(report: &QianjiBpmnWorkflowCancelReport) -> Self {
        Self {
            cancelled: true,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// JSON error body emitted by BPMN workflow HTTP routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowHttpErrorBody {
    /// Stable machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

#[derive(Debug)]
pub(super) struct QianjiBpmnWorkflowHttpError {
    status: StatusCode,
    body: QianjiBpmnWorkflowHttpErrorBody,
}

impl QianjiBpmnWorkflowHttpError {
    pub(super) fn bad_request(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: QianjiBpmnWorkflowHttpErrorBody {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

impl From<QianjiBpmnWorkflowControlError> for QianjiBpmnWorkflowHttpError {
    fn from(error: QianjiBpmnWorkflowControlError) -> Self {
        let (status, code) = match error {
            QianjiBpmnWorkflowControlError::CheckpointMissing { .. } => {
                (StatusCode::NOT_FOUND, "checkpoint_missing")
            }
            QianjiBpmnWorkflowControlError::Io(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "workflow_control_io")
            }
            QianjiBpmnWorkflowControlError::Orchestration(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "workflow_orchestration_failed",
            ),
        };
        Self {
            status,
            body: QianjiBpmnWorkflowHttpErrorBody {
                code: code.to_string(),
                message: error.to_string(),
            },
        }
    }
}

impl IntoResponse for QianjiBpmnWorkflowHttpError {
    fn into_response(self) -> Response {
        (self.status, axum::Json(self.body)).into_response()
    }
}
