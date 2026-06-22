//! Flowhub service-task scheduling adapters for `qianji-server` HTTP snapshots.

use xiuxian_qianji_bpmn_engine::PendingHostWork;
use xiuxian_qianji_control::{AdmittedActivityTaskScheduleRecord, ControlResult};
use xiuxian_qianji_runtime::{
    FlowhubServiceActivityScheduleInput, build_flowhub_service_activity_schedule_record,
};

use crate::bpmn::QianjiBpmnPendingHostWorkHttpResponse;

use super::types::FlowhubServiceActivityHttpScheduleInput;

/// Builds a durable `ActivityTask` schedule record from a qianji-server HTTP
/// pending service-work item.
///
/// # Errors
///
/// Returns a control error when the HTTP pending item does not describe a BPMN
/// service task, lacks BPMN identity, or contains invalid control-plane
/// identifiers.
pub fn build_flowhub_service_activity_schedule_record_from_http_pending_work(
    input: FlowhubServiceActivityHttpScheduleInput<'_>,
) -> ControlResult<AdmittedActivityTaskScheduleRecord> {
    let pending_work = pending_host_work_from_http_response(input.pending_work);
    build_flowhub_service_activity_schedule_record(FlowhubServiceActivityScheduleInput {
        run_id: input.run_id,
        occurred_at_ms: input.occurred_at_ms,
        scenario_id: input.scenario_id,
        instance_id: input.instance_id,
        bpmn_source: input.bpmn_source,
        pending_work: &pending_work,
    })
}

fn pending_host_work_from_http_response(
    work: &QianjiBpmnPendingHostWorkHttpResponse,
) -> PendingHostWork {
    PendingHostWork {
        token_id: work.token_id,
        process_id: work
            .process_id
            .as_ref()
            .map(|process_id| process_id.as_ref().into()),
        node_index: work.node_index,
        activity_id: work
            .activity_id
            .as_ref()
            .map(|activity_id| activity_id.as_ref().into()),
        kind: work.kind.clone(),
        decision: None,
        lane: work.lane.clone(),
        script_format: None,
        script_body: None,
        human_task_form: work.form.clone(),
        human_task_assignment: work.assignment.clone(),
        task_io: work.task_io.clone(),
        claim: work.claim.clone(),
        event_reference: None,
        event_name: None,
        work_id: work.work_id.as_ref().map(|work_id| work_id.as_str().into()),
    }
}
