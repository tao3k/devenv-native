//! Public runtime human-task claim entrypoint.

use crate::error::Result;
use crate::host_types_api::{BpmnHostActivityId, BpmnHostProcessId, BpmnHostTokenId};
use crate::runtime::{BpmnInstanceState, PendingHostWork};
use std::borrow::Borrow;

/// Unix timestamp in milliseconds for human-task claim operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnHumanTaskClaimedAtMs(u64);

impl BpmnHumanTaskClaimedAtMs {
    /// Returns the serialized timestamp.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BpmnHumanTaskClaimedAtMs {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Unix timestamp in milliseconds for human-task release operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnHumanTaskReleasedAtMs(u64);

impl BpmnHumanTaskReleasedAtMs {
    /// Returns the serialized timestamp.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BpmnHumanTaskReleasedAtMs {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Identity and claimant data for one human-task claim request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskClaimRequest {
    /// Runtime token identifier for the pending host work.
    pub token_id: BpmnHostTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: BpmnHostProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: BpmnHostActivityId,
    /// Host- or operator-facing claimant identifier.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the claim operation.
    pub claimed_at_ms: BpmnHumanTaskClaimedAtMs,
}

/// Identity and claimant data for one human-task release request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskReleaseRequest {
    /// Runtime token identifier for the pending host work.
    pub token_id: BpmnHostTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: BpmnHostProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: BpmnHostActivityId,
    /// Host- or operator-facing claimant identifier that currently owns the
    /// work.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the release operation.
    pub released_at_ms: BpmnHumanTaskReleasedAtMs,
}

/// Input for one human-task release request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskReleaseInput {
    /// Runtime token identifier for the pending host work.
    pub token_id: BpmnHostTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: BpmnHostProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: BpmnHostActivityId,
    /// Host- or operator-facing claimant identifier that currently owns the
    /// work.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the release operation.
    pub released_at_ms: BpmnHumanTaskReleasedAtMs,
}

/// Input for one human-task claim request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskClaimInput {
    /// Runtime token identifier for the pending host work.
    pub token_id: BpmnHostTokenId,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: BpmnHostProcessId,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: BpmnHostActivityId,
    /// Host- or operator-facing claimant identifier.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the claim operation.
    pub claimed_at_ms: BpmnHumanTaskClaimedAtMs,
}

impl PendingHumanTaskReleaseRequest {
    /// Creates one human-task release request.
    #[must_use]
    pub fn from_input(input: PendingHumanTaskReleaseInput) -> Self {
        Self {
            token_id: (input.token_id),
            process_id: (input.process_id),
            activity_id: (input.activity_id),
            claimant: input.claimant,
            released_at_ms: input.released_at_ms,
        }
    }
}

impl PendingHumanTaskClaimRequest {
    /// Creates one human-task claim request.
    #[must_use]
    pub fn from_input(input: PendingHumanTaskClaimInput) -> Self {
        Self {
            token_id: (input.token_id),
            process_id: (input.process_id),
            activity_id: (input.activity_id),
            claimant: input.claimant,
            claimed_at_ms: input.claimed_at_ms,
        }
    }
}

/// Result of one human-task claim operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskClaimOutcome {
    /// The pending host-work item after claim processing.
    pub pending_host_work: PendingHostWork,
    /// Whether the operation changed checkpointed runtime state.
    pub changed: bool,
}

/// Result of one human-task claim release operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskReleaseOutcome {
    /// The pending host-work item after release processing.
    pub pending_host_work: PendingHostWork,
    /// Whether the operation changed checkpointed runtime state.
    pub changed: bool,
}

/// Claims one currently pending BPMN `userTask` or `manualTask`.
///
/// # Errors
///
/// Returns [`BpmnEngineError`] when the pending work does not exist, the
/// process/activity identity does not match, the pending work is not human
/// work, the claimant is empty, or another claimant already owns the work.
pub fn claim_pending_human_task(
    instance: &mut BpmnInstanceState,
    request: impl Borrow<PendingHumanTaskClaimRequest>,
) -> Result<PendingHumanTaskClaimOutcome> {
    crate::runtime::claim_pending_human_task(instance, request)
}

/// Releases one currently claimed BPMN `userTask` or `manualTask`.
///
/// # Errors
///
/// Returns [`BpmnEngineError`] when the pending work does not exist, the
/// process/activity identity does not match, the pending work is not human
/// work, the claimant is empty, the work is unclaimed, or a different claimant
/// owns the work.
pub fn release_pending_human_task(
    instance: &mut BpmnInstanceState,
    request: impl Borrow<PendingHumanTaskReleaseRequest>,
) -> Result<PendingHumanTaskReleaseOutcome> {
    crate::runtime::release_pending_human_task(instance, request)
}
