use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStatusReport,
};
use crate::bpmn::driver::QianjiBpmnExecutionReport;
use qianji_bpmn_engine::{BpmnAdvanceOutcome, BpmnInstanceState, InstanceLifecycle};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    pub(in crate::bpmn::http_transport) fn from_start_report(
        report: &QianjiBpmnWorkflowStartReport,
    ) -> Self {
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
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowStatusReport,
    ) -> Self {
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
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowCancelReport,
    ) -> Self {
        Self {
            cancelled: true,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}
