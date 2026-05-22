use std::error::Error;
use std::io;

use xiuxian_qianji_control::{
    ActivityId, ActivityStatus, AdmittedLlmActivityScheduleRecord, ControlEvent, ControlEventKind,
    ControlLedger, InMemoryControlLedger, LlmActivityAdmission, LlmActivityRequest,
    LlmActivityTask, LlmModelId, RunId, StepId, record_admitted_llm_activity_schedule,
};

use super::support::llm_admission;
use crate::control::support::{activity_task, artifact_ref};

#[test]
fn helper_records_step_scoped_admitted_llm_activity_schedule_event() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-admitted-llm-schedule")?;
    let step_id = StepId::new("stage-admitted-llm")?;
    let activity_id = ActivityId::new("activity-admitted-llm")?;
    let admission = llm_admission(activity_id.clone())?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record admitted llm schedule".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "Admitted LLM stage".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    record_admitted_llm_activity_schedule(
        &ledger,
        AdmittedLlmActivityScheduleRecord::step(run_id.clone(), step_id.clone(), 3, admission),
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
    assert_eq!(
        activity
            .task
            .as_ref()
            .map(|task| task.activity_type.as_str()),
        Some("llm.plan")
    );
    assert_eq!(
        activity.task.as_ref().map(|task| task.task_queue.as_str()),
        Some("llm.openai")
    );
    let audit_metadata = &activity
        .task
        .as_ref()
        .ok_or_else(|| io::Error::other("missing replayed LLM task"))?
        .metadata["qianji_llm_activity_request"];
    assert_eq!(
        audit_metadata["schema"],
        "qianji.llm_activity_request_audit.v1"
    );
    assert_eq!(audit_metadata["model"], "openai/gpt-5.2");
    assert_eq!(
        audit_metadata["prompt_ref"]["artifact_id"],
        "artifact-llm-plan-input"
    );

    Ok(())
}

#[test]
fn llm_schedule_preserves_task_metadata_with_request_audit() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-llm-schedule-audit-metadata")?;
    let activity_id = ActivityId::new("activity-llm-schedule-audit-metadata")?;
    let mut task = activity_task(activity_id.clone())?;
    task.metadata = serde_json::json!({
        "scheduler": "qianji-control-test",
    });
    let prompt_ref = task
        .input_ref
        .clone()
        .ok_or_else(|| io::Error::other("missing prompt input ref"))?;
    let mut request = LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref)
        .with_context_ref(artifact_ref("artifact-llm-context")?)
        .with_tool_schema_hash("sha256:tool-schema")
        .with_max_tokens(2048)
        .with_response_schema_ref(artifact_ref("artifact-llm-response-schema")?);
    request.metadata = serde_json::json!({
        "prompt_version": "v1",
    });
    let admission = LlmActivityAdmission::from_activity(LlmActivityTask::new(task, request))?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "record admitted llm schedule audit metadata".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_admitted_llm_activity_schedule(
        &ledger,
        AdmittedLlmActivityScheduleRecord::run(run_id.clone(), 2, admission),
    )?;

    let view = ledger.load_run_view(&run_id)?;
    let activity = view
        .activities
        .get(&activity_id)
        .ok_or_else(|| io::Error::other("missing replayed activity"))?;
    let task = activity
        .task
        .as_ref()
        .ok_or_else(|| io::Error::other("missing replayed LLM task"))?;

    assert_eq!(task.metadata["scheduler"], "qianji-control-test");
    let audit_metadata = &task.metadata["qianji_llm_activity_request"];
    assert_eq!(
        audit_metadata["context_ref"]["artifact_id"],
        "artifact-llm-context"
    );
    assert_eq!(audit_metadata["tool_schema_hash"], "sha256:tool-schema");
    assert_eq!(audit_metadata["max_tokens"], 2048);
    assert_eq!(
        audit_metadata["response_schema_ref"]["artifact_id"],
        "artifact-llm-response-schema"
    );
    assert_eq!(audit_metadata["request_metadata"]["prompt_version"], "v1");

    Ok(())
}
