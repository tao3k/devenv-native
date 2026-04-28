use crate::bpmn::control::{
    QianjiBpmnWorkflowCancelReport, QianjiBpmnWorkflowStartReport, QianjiBpmnWorkflowStatusReport,
    QianjiBpmnWorkflowTaskClaimReport, QianjiBpmnWorkflowTaskReleaseReport,
};
use crate::bpmn::driver::QianjiBpmnExecutionReport;
use qianji_bpmn_engine::{
    BpmnAdvanceOutcome, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec,
    BpmnHumanTaskLifecycleEvent, BpmnInstanceState, BpmnLaneMembershipSpec, InstanceLifecycle,
    PendingHostWork, PendingHostWorkClaim, PendingHostWorkKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Pending host-work item embedded in HTTP workflow snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QianjiBpmnPendingHostWorkHttpResponse {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier for the pending host work.
    pub process_id: Option<String>,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN activity identifier for the blocked node.
    pub activity_id: Option<String>,
    /// Host work category.
    pub kind: PendingHostWorkKind,
    /// Optional host-generated work identifier.
    pub work_id: Option<String>,
    /// Optional human-task form metadata preserved for host rendering.
    pub form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN assignment metadata preserved for host routing.
    pub assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional BPMN lane membership metadata preserved for host routing.
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional checkpointed claim metadata.
    pub claim: Option<PendingHostWorkClaim>,
}

impl QianjiBpmnPendingHostWorkHttpResponse {
    fn from_pending_host_work(work: &PendingHostWork) -> Self {
        Self {
            token_id: work.token_id,
            process_id: work.process_id.clone(),
            node_index: work.node_index,
            activity_id: work.activity_id.clone(),
            kind: work.kind.clone(),
            work_id: work.work_id.clone(),
            form: work.human_task_form.clone(),
            assignment: work.human_task_assignment.clone(),
            lane: work.lane.clone(),
            claim: work.claim.clone(),
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
    /// Active host-work items with Rust-owned identity and human-task metadata.
    #[serde(default)]
    pub pending_host_work: Vec<QianjiBpmnPendingHostWorkHttpResponse>,
    /// Durable lifecycle events for BPMN `userTask` and `manualTask`.
    #[serde(default)]
    pub human_task_events: Vec<BpmnHumanTaskLifecycleEvent>,
    /// Number of active external wait registrations.
    pub wait_registration_count: usize,
    /// Number of active runtime tokens.
    pub active_token_count: usize,
}

impl QianjiBpmnWorkflowSnapshotHttpResponse {
    /// Creates one compact HTTP snapshot from an engine instance state.
    #[must_use]
    pub fn from_instance(instance: &BpmnInstanceState) -> Self {
        Self {
            instance_id: instance.instance_id.to_string(),
            process_id: instance.process.process_id.to_string(),
            sequence: instance.sequence,
            lifecycle: instance.lifecycle.clone(),
            variables: instance.variables.clone(),
            pending_host_work_count: instance.pending_host_work.len(),
            pending_host_work: instance
                .pending_host_work
                .iter()
                .map(QianjiBpmnPendingHostWorkHttpResponse::from_pending_host_work)
                .collect(),
            human_task_events: instance.human_task_events.clone(),
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

/// HTTP response for one checkpoint-backed BPMN human-task claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskClaimHttpResponse {
    /// Whether the claim mutated checkpointed state.
    pub claimed: bool,
    /// Monotonic checkpoint sequence after claim processing.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Claimed pending host-work item.
    pub claimed_work: QianjiBpmnPendingHostWorkHttpResponse,
    /// Runtime snapshot loaded after claim processing.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowTaskClaimHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowTaskClaimReport,
    ) -> Self {
        Self {
            claimed: report.changed,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            claimed_work: QianjiBpmnPendingHostWorkHttpResponse::from_pending_host_work(
                &report.claimed_work,
            ),
            workflow: QianjiBpmnWorkflowSnapshotHttpResponse::from_instance(&report.instance),
        }
    }
}

/// HTTP response for one checkpoint-backed BPMN human-task claim release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiBpmnWorkflowTaskReleaseHttpResponse {
    /// Whether the release mutated checkpointed state.
    pub released: bool,
    /// Monotonic checkpoint sequence after release processing.
    pub checkpoint_sequence: u64,
    /// Checkpoint backend used by the action.
    pub checkpoint_backend: String,
    /// Pending host-work item after release.
    pub released_work: QianjiBpmnPendingHostWorkHttpResponse,
    /// Runtime snapshot loaded after release processing.
    pub workflow: QianjiBpmnWorkflowSnapshotHttpResponse,
}

impl QianjiBpmnWorkflowTaskReleaseHttpResponse {
    pub(in crate::bpmn::http_transport) fn from_report(
        report: &QianjiBpmnWorkflowTaskReleaseReport,
    ) -> Self {
        Self {
            released: report.changed,
            checkpoint_sequence: report.checkpoint_sequence,
            checkpoint_backend: report.checkpoint_store.backend_name().to_string(),
            released_work: QianjiBpmnPendingHostWorkHttpResponse::from_pending_host_work(
                &report.released_work,
            ),
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
