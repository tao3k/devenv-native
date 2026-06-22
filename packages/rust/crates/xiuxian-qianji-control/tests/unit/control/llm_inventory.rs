use std::error::Error;

use xiuxian_qianji_control::{
    ActivityFailure, ActivityId, ActivityResult, ActivityStatus, ActivityTask, ActivityType,
    AdmittedLlmActivityScheduleRecord, ControlEvent, ControlEventKind, ControlLedger, ErrorCode,
    IdempotencyKey, InMemoryControlLedger, LlmActivityAdmission, LlmActivityRequest,
    LlmActivityTask, LlmModelId, RecoveryItemScope, RunId, StepId, TaskQueue,
    record_admitted_llm_activity_schedule,
};

use crate::control::support::{activity_task, artifact_ref};

#[test]
fn llm_activity_inventory_reports_status_and_request_audit_coverage() -> Result<(), Box<dyn Error>>
{
    let fixture = llm_inventory_fixture()?;

    let projection = fixture
        .ledger
        .load_llm_activity_inventory_projection(&fixture.run_id)?;

    assert_eq!(projection.run_id, fixture.run_id);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(projection.summary.total, 2);
    assert_eq!(projection.summary.completed, 1);
    assert_eq!(projection.summary.failed, 1);
    assert_eq!(projection.summary.missing_request_audit, 1);
    assert_eq!(projection.items[0].status, ActivityStatus::Completed);
    assert_eq!(projection.items[0].attempt, 1);
    assert_eq!(projection.items[0].updated_at_ms, 5);
    assert_eq!(projection.items[0].model.as_deref(), Some("openai/gpt-5.2"));
    assert_eq!(
        projection.items[0].request_audit_metadata["prompt_ref"]["artifact_id"],
        "artifact-llm-plan-input"
    );
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(fixture.step_id)
    );
    assert_eq!(projection.items[1].status, ActivityStatus::Failed);
    assert!(projection.items[1].model.is_none());
    assert!(projection.items[1].request_audit_metadata.is_null());

    Ok(())
}

struct LlmInventoryFixture {
    ledger: InMemoryControlLedger,
    run_id: RunId,
    step_id: StepId,
}

fn llm_inventory_fixture() -> Result<LlmInventoryFixture, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-llm-inventory")?;
    let step_id = StepId::new("step-llm-inventory")?;
    let audited_activity_id = ActivityId::new("activity-llm-audited")?;
    let missing_audit_activity_id = ActivityId::new("activity-llm-missing-audit")?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "inspect llm inventory".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        2,
        ControlEventKind::StepCreated {
            title: "LLM inventory step".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;

    record_admitted_llm_activity_schedule(
        &ledger,
        AdmittedLlmActivityScheduleRecord::run(
            run_id.clone(),
            3,
            llm_admission(audited_activity_id.clone())?,
        ),
    )?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        4,
        ControlEventKind::ActivityStarted {
            activity_id: audited_activity_id.clone(),
            worker_id: None,
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        5,
        ControlEventKind::ActivityCompleted {
            activity_id: audited_activity_id,
            result: ActivityResult {
                output_ref: Some(artifact_ref("artifact-llm-output")?),
                output_hash: Some("sha256:llm-output".to_owned()),
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        6,
        ControlEventKind::ActivityScheduled {
            task: legacy_llm_activity_task(&missing_audit_activity_id)?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        step_id.clone(),
        7,
        ControlEventKind::ActivityFailed {
            activity_id: missing_audit_activity_id,
            failure: ActivityFailure {
                error_code: ErrorCode::new("provider_failed")?,
                message: "provider failed".to_owned(),
                retryable: true,
                attempt: 1,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        8,
        ControlEventKind::ActivityScheduled {
            task: non_llm_activity_task(ActivityId::new("activity-tool-ignored")?)?,
        },
    ))?;
    Ok(LlmInventoryFixture {
        ledger,
        run_id,
        step_id,
    })
}

fn llm_admission(activity_id: ActivityId) -> Result<LlmActivityAdmission, Box<dyn Error>> {
    let task = activity_task(activity_id)?;
    let prompt_ref = task
        .input_ref
        .clone()
        .ok_or_else(|| std::io::Error::other("missing prompt input ref"))?;
    Ok(LlmActivityAdmission::from_activity(LlmActivityTask::new(
        task,
        LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref),
    ))?)
}

fn legacy_llm_activity_task(activity_id: &ActivityId) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        activity_id.clone(),
        ActivityType::new("llm.repair")?,
        TaskQueue::new("llm.openrouter")?,
        IdempotencyKey::new(format!("{activity_id}/legacy"))?,
    )
    .with_input_ref(artifact_ref("artifact-legacy-llm-input")?)
    .with_timeout_ms(30_000))
}

fn non_llm_activity_task(activity_id: ActivityId) -> Result<ActivityTask, Box<dyn Error>> {
    Ok(ActivityTask::new(
        activity_id,
        ActivityType::new("tool.web")?,
        TaskQueue::new("tool.web")?,
        IdempotencyKey::new("activity-tool-ignored/key")?,
    )
    .with_input_ref(artifact_ref("artifact-tool-input")?)
    .with_timeout_ms(30_000))
}
