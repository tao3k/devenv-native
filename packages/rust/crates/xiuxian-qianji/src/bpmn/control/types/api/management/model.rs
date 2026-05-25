//! Status, listing, cancel, and interrupt contracts for BPMN control.

use super::execution::QianjiBpmnWorkflowCheckpointBackend;
use super::human_work::QianjiBpmnWorkflowWorklistItem;
use crate::bpmn::backend::QianjiBpmnCheckpointStore;
use crate::bpmn::identity::{
    QianjiBpmnPackageId, QianjiBpmnProcessId, QianjiBpmnWorkflowInstanceId,
};
use xiuxian_qianji_bpmn_engine::{BpmnInstanceState, PendingHostWork};

/// Typed request for loading one checkpoint-backed BPMN workflow status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowStatusRequest {
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Checkpoint backend to inspect for this bounded status request.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for listing checkpoint-backed BPMN workflow instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInstancesRequest {
    /// Checkpoint backend to inspect for this bounded instance-list request.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for canceling one checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowCancelRequest {
    /// Workflow instance identifier used for checkpoint lookup and deletion.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Checkpoint backend to cancel for this bounded workflow instance.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Typed request for interrupting one checkpoint-backed BPMN workflow instance
/// while preserving durable checkpoint state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInterruptRequest {
    /// Workflow instance identifier used for checkpoint lookup and preservation.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// Checkpoint backend to interrupt for this bounded workflow instance.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow status load.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowStatusReport {
    /// Resolved checkpoint store used for this status request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state stored in the checkpoint payload.
    pub instance: BpmnInstanceState,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN human-task claim.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowTaskClaimReport {
    /// Resolved checkpoint store used for this claim request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence persisted after the claim.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state persisted after the claim.
    pub instance: BpmnInstanceState,
    /// Claimed pending host-work item after claim processing.
    pub claimed_work: PendingHostWork,
    /// Whether the claim mutated checkpointed state.
    pub changed: bool,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN human-task claim release.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowTaskReleaseReport {
    /// Resolved checkpoint store used for this release request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence persisted after the release.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state persisted after the release.
    pub instance: BpmnInstanceState,
    /// Pending host-work item after release processing.
    pub released_work: PendingHostWork,
    /// Whether the release mutated checkpointed state.
    pub changed: bool,
}

/// Report returned by the workflow control service after listing checkpointed
/// pending human work.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowWorklistReport {
    /// Resolved checkpoint store used for this worklist request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Pending human-work items derived from checkpointed engine state.
    pub work_items: Vec<QianjiBpmnWorkflowWorklistItem>,
}

/// Compact checkpoint summary for one persisted BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowInstanceSummary {
    /// Workflow instance identifier.
    pub instance_id: QianjiBpmnWorkflowInstanceId,
    /// BPMN process identifier.
    pub process_id: QianjiBpmnProcessId,
    /// BPMN package identifier.
    pub package_id: QianjiBpmnPackageId,
    /// Durable instance lifecycle.
    pub lifecycle: xiuxian_qianji_bpmn_engine::InstanceLifecycle,
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Engine state sequence inside the checkpoint payload.
    pub state_sequence: u64,
    /// Last checkpoint update timestamp in unix milliseconds.
    pub updated_at_ms: u64,
    /// Number of active runtime tokens.
    pub active_token_count: usize,
    /// Number of pending host-work entries.
    pub pending_host_work_count: usize,
    /// Number of registered waits.
    pub wait_registration_count: usize,
}

impl QianjiBpmnWorkflowInstanceSummary {
    pub(crate) fn from_checkpoint(
        checkpoint: xiuxian_qianji_bpmn_engine::BpmnCheckpointEnvelope,
    ) -> Self {
        Self {
            instance_id: checkpoint.state.instance_id.as_ref().into(),
            process_id: checkpoint.state.process.process_id.as_ref().into(),
            package_id: checkpoint.state.process.package_id.as_ref().into(),
            lifecycle: checkpoint.state.lifecycle,
            checkpoint_sequence: checkpoint.sequence,
            state_sequence: checkpoint.state.sequence,
            updated_at_ms: checkpoint.state.updated_at_ms,
            active_token_count: checkpoint.state.active_tokens.len(),
            pending_host_work_count: checkpoint.state.pending_host_work.len(),
            wait_registration_count: checkpoint.state.waits.len(),
        }
    }
}

/// Report returned by the workflow control service after listing checkpointed
/// BPMN workflow instances.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowInstancesReport {
    /// Resolved checkpoint store used for this instance-list request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Latest known checkpoint summaries, newest first when supported by the backend.
    pub instances: Vec<QianjiBpmnWorkflowInstanceSummary>,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow cancellation.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowCancelReport {
    /// Resolved checkpoint store used for this cancel request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence loaded before deletion.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state loaded before deletion.
    pub instance: BpmnInstanceState,
}

/// Report returned by the workflow control service after one checkpoint-first
/// BPMN workflow interruption.
#[derive(Debug, Clone)]
pub struct QianjiBpmnWorkflowInterruptReport {
    /// Resolved checkpoint store used for this interrupt request.
    pub checkpoint_store: QianjiBpmnCheckpointStore,
    /// Monotonic checkpoint sequence persisted after interruption.
    pub checkpoint_sequence: u64,
    /// Durable BPMN instance state persisted after interruption.
    pub instance: BpmnInstanceState,
}
