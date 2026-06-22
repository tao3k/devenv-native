use std::error::Error;

use serde_json::json;
use xiuxian_qianji_bpmn_engine::PendingHostWorkKind;
use xiuxian_qianji_control::RunId;
use xiuxian_qianji_runtime::{
    BPMN_HOST_WORK_ACTIVITY_TYPE, build_bpmn_host_work_activity_schedule_record,
};

use super::support::{adapter_input, pending_work};

#[test]
fn bpmn_host_work_schedule_record_preserves_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work")?;
    let work = pending_work(PendingHostWorkKind::Service);
    let record = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(record.run_id, run_id);
    assert_eq!(record.occurred_at_ms, 77);
    assert_eq!(
        record.task.activity_type.as_str(),
        BPMN_HOST_WORK_ACTIVITY_TYPE
    );
    assert_eq!(record.task.task_queue.as_str(), "bpmn.host_work.service");
    assert_eq!(
        record.task.activity_id.as_str(),
        "bpmn.instance_1.Process_1.Task_Review.9"
    );
    assert_eq!(
        record.task.idempotency_key.as_str(),
        "idempotency.bpmn.instance_1.Process_1.Task_Review.9"
    );
    assert_eq!(
        record
            .task
            .input_ref
            .as_ref()
            .map(|input| input.uri.as_str()),
        Some("bpmn://instances/instance_1/processes/Process_1/tokens/9/host-work/Task_Review")
    );

    let metadata = &record.task.metadata["qianji_bpmn_host_work_activity"];
    assert_eq!(
        metadata["schema"],
        "xiuxian_qianji.bpmn.host_work_activity.v1"
    );
    assert_eq!(metadata["instanceId"], "instance_1");
    assert_eq!(metadata["processId"], "Process_1");
    assert_eq!(metadata["activityId"], "Task_Review");
    assert_eq!(metadata["tokenId"], 9);
    assert_eq!(metadata["workKind"], "service");
    assert_eq!(
        metadata["requiredOutputs"],
        json!([{
            "name": "approved",
            "targetRef": "review.approved",
            "required": true
        }])
    );

    Ok(())
}

#[test]
fn bpmn_host_work_schedule_record_supports_business_rule_work() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work-business-rule")?;
    let work = pending_work(PendingHostWorkKind::BusinessRule);
    let record = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(
        record.task.task_queue.as_str(),
        "bpmn.host_work.business_rule"
    );
    assert_eq!(
        record.task.metadata["qianji_bpmn_host_work_activity"]["workKind"],
        "business_rule"
    );
    Ok(())
}

#[test]
fn bpmn_host_work_schedule_record_requires_bpmn_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work-missing-id")?;
    let mut work = pending_work(PendingHostWorkKind::User);
    work.process_id = None;

    let Err(error) = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))
    else {
        return Err("missing process id should be rejected".into());
    };

    assert!(error.to_string().contains("requires a process id"));
    Ok(())
}
