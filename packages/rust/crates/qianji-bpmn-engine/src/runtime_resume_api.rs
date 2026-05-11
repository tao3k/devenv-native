//! Public runtime host-resume entrypoint.

use crate::error::Result;
use crate::host_types_api::PendingHostWorkResult;
use crate::ir::BpmnPackage;
use crate::runtime::BpmnInstanceState;
use crate::runtime_advance_api::BpmnAdvanceOutcome;

/// Public runtime token identifier used by host-resume APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BpmnHostWorkTokenId(u64);

impl BpmnHostWorkTokenId {
    /// Returns the serialized runtime token identifier.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BpmnHostWorkTokenId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Unix timestamp in milliseconds for one host-work completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct BpmnHostWorkCompletedAtMs(u64);

impl BpmnHostWorkCompletedAtMs {
    /// Returns the serialized timestamp.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for BpmnHostWorkCompletedAtMs {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Input for applying one host-side completion result.
pub struct PendingHostWorkApplyInput<'a> {
    /// BPMN package containing the process model.
    pub package: &'a BpmnPackage,
    /// Mutable runtime instance state.
    pub instance: &'a mut BpmnInstanceState,
    /// Runtime token identifier for the pending host work.
    pub token_id: BpmnHostWorkTokenId,
    /// Host-side completion payload.
    pub result: PendingHostWorkResult,
    /// Unix timestamp in milliseconds for the completion operation.
    pub completed_at_ms: BpmnHostWorkCompletedAtMs,
}

/// Applies one host-side completion result to the currently blocked BPMN
/// instance and resumes local routing state.
///
/// # Identifier Boundary
///
/// The `token_id` primitive is kept at this public boundary because host
/// completion payloads refer to the serialized runtime token id.
///
/// # Errors
///
/// Returns [`BpmnEngineError::MissingPendingHostWork`] when the instance is not
/// currently blocked on host work, [`BpmnEngineError::HostResultKindMismatch`]
/// when the supplied host result does not match the pending work kind, or
/// [`BpmnEngineError`] when the process/model shape exceeds the supported
/// bounded subset.
pub fn apply_pending_host_work_result(
    input: PendingHostWorkApplyInput<'_>,
) -> Result<BpmnAdvanceOutcome> {
    crate::runtime::apply_pending_host_work_result(
        input.package,
        input.instance,
        input.token_id.get(),
        input.result,
        input.completed_at_ms.get(),
    )
}
