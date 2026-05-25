use std::path::Path;

use xiuxian_qianji_control::RunId;

use crate::bpmn::QianjiBpmnPendingHostWorkHttpResponse;

/// Input for converting one qianji-server HTTP pending service boundary into
/// an `ActivityTask` schedule record.
#[derive(Debug, Clone, Copy)]
pub struct FlowhubServiceActivityHttpScheduleInput<'a> {
    /// Owning Qianji control-plane run id.
    pub run_id: &'a RunId,
    /// Schedule timestamp supplied by the caller.
    pub occurred_at_ms: u64,
    /// Flowhub scenario id, for example `agent-coding`.
    pub scenario_id: &'a str,
    /// BPMN workflow instance id.
    pub instance_id: &'a str,
    /// Source BPMN document path used by the workflow route.
    pub bpmn_source: &'a Path,
    /// Pending BPMN host work returned by qianji-server HTTP snapshots.
    pub pending_work: &'a QianjiBpmnPendingHostWorkHttpResponse,
}
