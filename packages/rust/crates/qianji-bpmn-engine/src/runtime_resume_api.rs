//! Public runtime host-resume entrypoint.

use crate::error::Result;
use crate::host_types_api::PendingHostWorkResult;
use crate::ir::BpmnPackage;
use crate::runtime::BpmnInstanceState;
use crate::runtime_advance_api::BpmnAdvanceOutcome;
use std::borrow::Borrow;

/// Applies one host-side completion result to the currently blocked BPMN
/// instance and resumes local routing state.
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
    token_id: u64,
    result: impl Borrow<PendingHostWorkResult>,
    completed_at_ms: u64,
) -> Result<BpmnAdvanceOutcome> {
    crate::runtime::apply_pending_host_work_result(
        package,
        instance,
        token_id,
        result,
        completed_at_ms,
    )
}
