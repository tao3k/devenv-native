use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityJournalWriteStatus, ActivityStatus, AdmittedActivityScheduleRecord,
    AgentProposalId, ControlEvent, ControlEventKind, ControlLedger, InMemoryControlLedger, RunId,
    StepId, ToolActivityAdmission, ToolName, record_admitted_activity_schedule,
    record_admitted_activity_schedule_idempotent,
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

#[test]
fn idempotent_helper_returns_existing_activity_schedule_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-idempotent-admitted-schedule")?;
    let step_id = StepId::new("stage-idempotent-tool")?;
    let proposal_id = AgentProposalId::new("proposal-idempotent-tool")?;
    let activity_id = ActivityId::new("activity-idempotent-tool")?;
    let admission = ToolActivityAdmission {
        proposal_id,
        tool_name: ToolName::new("web.fetch")?,
        task: activity_task(activity_id)?,
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record idempotent admitted activity schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Idempotent admitted tool stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    let request =
        AdmittedActivityScheduleRecord::step(run_id.clone(), step_id.clone(), 3, admission);
    let appended = record_admitted_activity_schedule_idempotent(&ledger, request.clone())?;
    let duplicate = record_admitted_activity_schedule_idempotent(&ledger, request)?;

    assert_eq!(appended.status, ActivityJournalWriteStatus::Appended);
    assert_eq!(
        duplicate.status,
        ActivityJournalWriteStatus::AlreadyRecorded
    );
    assert_eq!(duplicate.record.sequence, appended.record.sequence);
    assert_eq!(ledger.load_events(&run_id)?.len(), 3);

    Ok(())
}

#[test]
fn idempotent_schedule_rejects_conflicting_activity_schedule() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-conflicting-admitted-schedule")?;
    let activity_id = ActivityId::new("activity-conflicting-tool")?;
    let original_admission = ToolActivityAdmission {
        proposal_id: AgentProposalId::new("proposal-original-tool")?,
        tool_name: ToolName::new("web.fetch")?,
        task: activity_task(activity_id.clone())?,
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };
    let conflicting_admission = ToolActivityAdmission {
        proposal_id: AgentProposalId::new("proposal-conflicting-tool")?,
        tool_name: ToolName::new("web.fetch")?,
        task: activity_task(activity_id)?.with_timeout_ms(45_000),
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "reject conflicting admitted activity schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_admitted_activity_schedule_idempotent(
        &ledger,
        AdmittedActivityScheduleRecord::run(run_id.clone(), 2, original_admission),
    )?;

    let Err(error) = record_admitted_activity_schedule_idempotent(
        &ledger,
        AdmittedActivityScheduleRecord::run(run_id.clone(), 3, conflicting_admission),
    ) else {
        return Err(io::Error::other("conflicting schedule should fail").into());
    };

    assert!(
        error.to_string().contains("already exists"),
        "unexpected error: {error}"
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 2);

    Ok(())
}

#[test]
fn helper_rejects_invalid_admitted_activity_schedule_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-invalid-admitted-schedule")?;
    let admission = ToolActivityAdmission {
        proposal_id: AgentProposalId::new("proposal-invalid-admitted-tool")?,
        tool_name: ToolName::new("web.fetch")?,
        task: activity_task(ActivityId::new("activity-invalid-admitted-tool")?)?.with_timeout_ms(0),
        approval_request_id: None,
        metadata: serde_json::Value::Null,
    };

    let Err(error) = record_admitted_activity_schedule(
        &ledger,
        AdmittedActivityScheduleRecord::run(run_id.clone(), 1, admission),
    ) else {
        return Err(io::Error::other("invalid admitted activity should fail").into());
    };

    assert!(
        error.to_string().contains("timeout_ms"),
        "unexpected error: {error}"
    );
    assert!(ledger.load_events(&run_id)?.is_empty());

    Ok(())
}
