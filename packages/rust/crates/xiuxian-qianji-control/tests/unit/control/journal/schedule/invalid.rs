use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, AgentProposalId, ControlLedger, InMemoryControlLedger, LlmActivityAdmission,
    LlmActivityRequest, LlmActivityTask, LlmModelId, RunId, ToolActivityAdmission, ToolName,
    record_admitted_activity_schedule, record_admitted_llm_activity_schedule,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn helper_rejects_invalid_admitted_llm_activity_schedule_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-invalid-admitted-llm-schedule")?;
    let task = activity_task(ActivityId::new("activity-invalid-admitted-llm")?)?
        .with_input_ref(artifact_ref("artifact-other-prompt")?);
    let admission = LlmActivityAdmission {
        activity: LlmActivityTask::new(
            task,
            LlmActivityRequest::new(
                LlmModelId::new("openai/gpt-5.2")?,
                artifact_ref("artifact-llm-plan-input")?,
            ),
        ),
        metadata: serde_json::Value::Null,
    };

    let Err(error) = record_admitted_llm_activity_schedule(
        &ledger,
        xiuxian_qianji_control::AdmittedLlmActivityScheduleRecord::run(
            run_id.clone(),
            1,
            admission,
        ),
    ) else {
        return Err(io::Error::other("invalid admitted llm activity should fail").into());
    };

    assert!(
        error.to_string().contains("prompt_ref"),
        "unexpected error: {error}"
    );
    assert!(ledger.load_events(&run_id)?.is_empty());

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
        xiuxian_qianji_control::AdmittedActivityScheduleRecord::run(run_id.clone(), 1, admission),
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
