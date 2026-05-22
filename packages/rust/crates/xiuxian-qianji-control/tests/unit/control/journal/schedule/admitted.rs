use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityStatus, AdmittedActivityScheduleRecord, AgentProposalId, ControlEvent,
    ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, StepId, ToolActivityAdmission,
    ToolName, record_admitted_activity_schedule,
};

use crate::control::support::activity_task;

#[test]
fn helper_records_step_scoped_admitted_activity_schedule_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-admitted-activity-schedule")?;
    let step_id = StepId::new("stage-admitted-tool")?;
    let proposal_id = AgentProposalId::new("proposal-admitted-tool")?;
    let activity_id = ActivityId::new("activity-admitted-tool")?;
    let task = activity_task(activity_id.clone())?;
    let admission = ToolActivityAdmission {
        proposal_id,
        tool_name: ToolName::new("web.fetch")?,
        task,
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record admitted activity schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Admitted tool stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    record_admitted_activity_schedule(
        &ledger,
        AdmittedActivityScheduleRecord::step(run_id.clone(), step_id.clone(), 3, admission),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let step = view
        .steps
        .get(&step_id)
        .ok_or_else(|| io::Error::other("missing replayed step"))?;
    let activity = step
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;

    assert_eq!(activity.status, ActivityStatus::Scheduled);
    assert_eq!(activity.worker_id, None);
    assert_eq!(activity.attempt, 0);
    assert_eq!(
        activity.task.as_ref().map(|task| &task.activity_id),
        Some(&activity_id)
    );
    assert!(step.active_lease.is_none());

    Ok(())
}
