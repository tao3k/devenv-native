use xiuxian_qianji_bpmn_engine::{PendingHostWork, PendingHostWorkKind};

use crate::flowhub::{
    QianjiRuntimeBpmnActivityId, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
};

/// Runtime-neutral identity for one checkpointed BPMN host-work boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnHostWorkIdentity {
    /// Runtime token identifier for the pending host work.
    pub token_id: QianjiRuntimeBpmnTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: QianjiRuntimeBpmnProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: QianjiRuntimeBpmnActivityId,
    /// Host work category expected for the pending host work.
    pub kind: PendingHostWorkKind,
}

impl BpmnHostWorkIdentity {
    /// Creates one runtime-neutral BPMN host-work identity.
    #[must_use]
    pub fn new(
        token_id: QianjiRuntimeBpmnTokenId,
        process_id: QianjiRuntimeBpmnProcessId,
        activity_id: QianjiRuntimeBpmnActivityId,
        kind: PendingHostWorkKind,
    ) -> Self {
        Self {
            token_id,
            process_id,
            activity_id,
            kind,
        }
    }
}

/// Finds the pending host work that matches a runtime-neutral identity.
#[must_use]
pub fn find_matching_bpmn_host_work<'a>(
    pending_host_work: &'a [PendingHostWork],
    identity: &BpmnHostWorkIdentity,
) -> Option<&'a PendingHostWork> {
    pending_host_work
        .iter()
        .find(|work| pending_bpmn_host_work_matches_identity(work, identity))
}

/// Returns whether one checkpointed pending host-work item matches identity.
#[must_use]
pub fn pending_bpmn_host_work_matches_identity(
    work: &PendingHostWork,
    identity: &BpmnHostWorkIdentity,
) -> bool {
    work.token_id == identity.token_id.as_u64()
        && work
            .process_id
            .as_ref()
            .is_some_and(|process_id| process_id.as_str() == identity.process_id.as_str())
        && work
            .activity_id
            .as_ref()
            .is_some_and(|activity_id| activity_id.as_str() == identity.activity_id.as_str())
        && work.kind == identity.kind
}
