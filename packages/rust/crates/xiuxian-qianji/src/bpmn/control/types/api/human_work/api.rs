use super::execution::QianjiBpmnWorkflowCheckpointBackend;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, BpmnHumanTaskAssignmentSpec, BpmnHumanTaskFormSpec,
    BpmnLaneMembershipSpec, BpmnTaskIoSpec, PendingHostWork, PendingHostWorkClaim,
    PendingHostWorkKind,
};

/// Explicit payload for claiming pending human work on one checkpoint-backed
/// BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowTaskClaimPayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Host- or operator-facing claimant identifier.
    pub claimant: String,
}

/// Typed request for claiming one pending human task on a checkpoint-backed
/// BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowTaskClaimRequest {
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Explicit human-task claim payload.
    pub claim: QianjiBpmnWorkflowTaskClaimPayload,
}

/// Explicit payload for releasing a pending human-work claim on one
/// checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowTaskReleasePayload {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Host- or operator-facing claimant identifier that currently owns the
    /// work.
    pub claimant: String,
}

/// Typed request for releasing one pending human-task claim on a
/// checkpoint-backed BPMN workflow instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowTaskReleaseRequest {
    /// Workflow instance identifier used for checkpoint lookup.
    pub instance_id: String,
    /// Checkpoint backend that already owns persisted workflow state.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Explicit human-task claim release payload.
    pub release: QianjiBpmnWorkflowTaskReleasePayload,
}

/// Typed request for listing checkpoint-backed pending human work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowWorklistRequest {
    /// Checkpoint backend to inspect for this bounded worklist request.
    pub checkpoint_backend: QianjiBpmnWorkflowCheckpointBackend,
    /// Optional claimant filter. When present, returns unclaimed human work and
    /// work already claimed by that same claimant.
    pub claimant: Option<String>,
    /// Optional passive assignment routing filters.
    pub routing: QianjiBpmnWorkflowWorklistRoutingFilter,
}

/// Passive worklist routing filters derived from Rust-owned BPMN metadata.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QianjiBpmnWorkflowWorklistRoutingFilter {
    /// Optional assignment resource filter. Matches standard BPMN
    /// `humanPerformer` or `potentialOwner` role names and `resourceRef`
    /// values exactly after trimming surrounding whitespace.
    pub assignment_resource: Option<String>,
    /// Optional passive BPMN lane filter. Matches lane id or lane name exactly
    /// after trimming surrounding whitespace.
    pub lane: Option<String>,
}

/// Compact pending human-work item derived from checkpointed engine state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnWorkflowWorklistItem {
    /// Workflow instance identifier.
    pub instance_id: String,
    /// BPMN process identifier for the pending host work.
    pub process_id: String,
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN node index.
    pub node_index: u32,
    /// Stable BPMN activity identifier for the blocked node.
    pub activity_id: String,
    /// Host work category.
    pub kind: PendingHostWorkKind,
    /// Optional human-task form metadata preserved for host rendering.
    pub form: Option<BpmnHumanTaskFormSpec>,
    /// Optional standard BPMN assignment metadata preserved for host routing.
    pub assignment: Option<BpmnHumanTaskAssignmentSpec>,
    /// Optional BPMN lane membership metadata preserved for host routing.
    pub lane: Option<BpmnLaneMembershipSpec>,
    /// Optional standard BPMN task IO metadata preserved for host routing.
    pub task_io: Option<BpmnTaskIoSpec>,
    /// Optional checkpointed claim metadata.
    pub claim: Option<PendingHostWorkClaim>,
    /// Monotonic checkpoint sequence loaded from the persisted envelope.
    pub checkpoint_sequence: u64,
    /// Engine state sequence inside the checkpoint payload.
    pub state_sequence: u64,
    /// Last checkpoint update timestamp in unix milliseconds.
    pub updated_at_ms: u64,
}

impl QianjiBpmnWorkflowWorklistItem {
    pub(crate) fn from_pending_host_work(
        checkpoint: &BpmnCheckpointEnvelope,
        pending: &PendingHostWork,
    ) -> Option<Self> {
        if !matches!(
            pending.kind,
            PendingHostWorkKind::User | PendingHostWorkKind::Manual
        ) {
            return None;
        }
        let process_id = pending.process_id.as_ref().map_or_else(
            || checkpoint.state.process.process_id.to_string(),
            |process_id| process_id.as_str().to_owned(),
        );
        let activity_id = pending.activity_id.as_ref().map_or_else(
            || format!("node#{}", pending.node_index),
            |activity_id| activity_id.as_str().to_owned(),
        );

        Some(Self {
            instance_id: checkpoint.state.instance_id.to_string(),
            process_id,
            token_id: pending.token_id,
            node_index: pending.node_index,
            activity_id,
            kind: pending.kind.clone(),
            form: pending.human_task_form.clone(),
            assignment: pending.human_task_assignment.clone(),
            lane: pending.lane.clone(),
            task_io: pending.task_io.clone(),
            claim: pending.claim.clone(),
            checkpoint_sequence: checkpoint.sequence,
            state_sequence: checkpoint.state.sequence,
            updated_at_ms: checkpoint.state.updated_at_ms,
        })
    }
}
