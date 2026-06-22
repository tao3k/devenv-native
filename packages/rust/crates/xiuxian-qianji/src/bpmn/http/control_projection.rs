//! Shared qianji-server control projection for BPMN HTTP runs.

use super::control_trace::record_bpmn_control_trace;
use super::error_api::QianjiBpmnWorkflowHttpError;
use super::llm_host_work_schedule::record_bpmn_llm_host_work_schedules;
use super::state::QianjiBpmnWorkflowHttpState;
use crate::bpmn::session::QianjiBpmnSession;
use std::path::Path;
use xiuxian_qianji_bpmn_engine::BpmnHostBridge;

pub(super) fn record_bpmn_control_projection<H>(
    state: &QianjiBpmnWorkflowHttpState<H>,
    session: &QianjiBpmnSession,
    bpmn_source: Option<&Path>,
) -> Result<(), QianjiBpmnWorkflowHttpError>
where
    H: BpmnHostBridge + Clone + Send + Sync + 'static,
{
    record_bpmn_control_trace(
        state.activity_evidence_ledger.as_deref(),
        session,
        bpmn_source,
    )?;
    if state.runtime_env.is_none() {
        return Ok(());
    }
    record_bpmn_llm_host_work_schedules(
        state.activity_evidence_ledger.as_deref(),
        state.runtime_env.as_ref(),
        session,
        bpmn_source,
    )
}
