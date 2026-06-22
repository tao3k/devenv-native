use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityJournalWriteStatus, AdmittedActivityScheduleRecord,
    AdmittedLlmActivityScheduleRecord, AgentProposalId, ControlEvent, ControlEventKind,
    ControlLedger, InMemoryControlLedger, LlmActivityAdmission, LlmActivityRequest,
    LlmActivityTask, LlmModelId, RunId, StepId, ToolActivityAdmission, ToolName,
    record_admitted_activity_schedule_idempotent, record_admitted_llm_activity_schedule_idempotent,
};

use super::support::llm_admission;
use crate::control::support::activity_task;

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
fn idempotent_llm_schedule_returns_existing_activity_schedule_event() -> Result<(), Box<dyn Error>>
{
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-idempotent-llm-schedule")?;
    let step_id = StepId::new("stage-idempotent-llm")?;
    let admission = llm_admission(ActivityId::new("activity-idempotent-llm")?)?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record idempotent admitted llm schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Idempotent admitted LLM stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    let request =
        AdmittedLlmActivityScheduleRecord::step(run_id.clone(), step_id.clone(), 3, admission);
    let appended = record_admitted_llm_activity_schedule_idempotent(&ledger, request.clone())?;
    let duplicate = record_admitted_llm_activity_schedule_idempotent(&ledger, request)?;

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
fn idempotent_llm_schedule_rejects_conflicting_activity_schedule() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-conflicting-admitted-llm-schedule")?;
    let activity_id = ActivityId::new("activity-conflicting-llm")?;
    let original_admission = llm_admission(activity_id.clone())?;
    let conflicting_task = activity_task(activity_id)?.with_timeout_ms(45_000);
    let prompt_ref = conflicting_task
        .input_ref
        .clone()
        .ok_or_else(|| io::Error::other("missing conflicting prompt input ref"))?;
    let conflicting_admission = LlmActivityAdmission::from_activity(LlmActivityTask::new(
        conflicting_task,
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref),
    ))?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "reject conflicting admitted llm schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_admitted_llm_activity_schedule_idempotent(
        &ledger,
        AdmittedLlmActivityScheduleRecord::run(run_id.clone(), 2, original_admission),
    )?;

    let Err(error) = record_admitted_llm_activity_schedule_idempotent(
        &ledger,
        AdmittedLlmActivityScheduleRecord::run(run_id.clone(), 3, conflicting_admission),
    ) else {
        return Err(io::Error::other("conflicting llm schedule should fail").into());
    };

    assert!(
        error.to_string().contains("already exists"),
        "unexpected error: {error}"
    );
    assert_eq!(ledger.load_events(&run_id)?.len(), 2);

    Ok(())
}
