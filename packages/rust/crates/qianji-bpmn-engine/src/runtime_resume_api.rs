//! Public runtime host-resume entrypoint.

use crate::error::Result;
use crate::host_types_api::PendingHostWorkResult;
use crate::ir::BpmnPackage;
use crate::runtime::BpmnInstanceState;
use crate::runtime_advance_api::BpmnAdvanceOutcome;
use std::borrow::Borrow;

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
    package: &BpmnPackage,
    instance: &mut BpmnInstanceState,
    token_id: impl Into<BpmnHostWorkTokenId>,
    result: impl Borrow<PendingHostWorkResult>,
    completed_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    let token_id = token_id.into();
    crate::runtime::apply_pending_host_work_result(
        package,
        instance,
        token_id.get(),
        result,
        completed_at_ms,
    )
}
