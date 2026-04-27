//! Public runtime human-task claim entrypoint.

use crate::error::Result;
use crate::runtime::{BpmnInstanceState, PendingHostWork};
use std::borrow::Borrow;

/// Identity and claimant data for one human-task claim request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskClaimRequest {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Host- or operator-facing claimant identifier.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the claim operation.
    pub claimed_at_ms: u64,
}

/// Identity and claimant data for one human-task release request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingHumanTaskReleaseRequest {
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Host- or operator-facing claimant identifier that currently owns the
    /// work.
    pub claimant: String,
    /// Unix timestamp in milliseconds for the release operation.
    pub released_at_ms: u64,
}

impl PendingHumanTaskReleaseRequest {
    /// Creates one human-task release request.
    #[must_use]
    pub fn new(
        token_id: u64,
        process_id: impl Into<String>,
        activity_id: impl Into<String>,
        claimant: impl Into<String>,
        released_at_ms: u64,
    ) -> Self {
        Self {
            token_id,
            process_id: process_id.into(),
            activity_id: activity_id.into(),
            claimant: claimant.into(),
            released_at_ms,
        }
    }
}

impl PendingHumanTaskClaimRequest {
    /// Creates one human-task claim request.
    #[must_use]
    pub fn new(
        token_id: u64,
        process_id: impl Into<String>,
        activity_id: impl Into<String>,
        claimant: impl Into<String>,
        claimed_at_ms: u64,
    ) -> Self {
        Self {
            token_id,
            process_id: process_id.into(),
            activity_id: activity_id.into(),
            claimant: claimant.into(),
            claimed_at_ms,
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
