use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ControlError, IdempotencyKey, LlmActivityRequest,
    LlmActivityTask, LlmModelId, TaskQueue,
};

use crate::control::support::artifact_ref;

#[test]
fn llm_activity_request_contract_rejects_invalid_payloads() -> Result<(), Box<dyn Error>> {
    let valid_request = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2")?,
        artifact_ref("artifact-llm-prompt")?,
    )
    .with_context_ref(artifact_ref("artifact-llm-context")?)
    .with_tool_schema_hash("sha256:tool-schema")
    .with_temperature_millis(200)
    .with_max_tokens(1_024)
    .with_response_schema_ref(artifact_ref("artifact-response-schema")?);
    valid_request.validate()?;

    let zero_tokens = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2")?,
        artifact_ref("artifact-llm-prompt-zero")?,
    )
    .with_max_tokens(0);
    assert!(matches!(
        zero_tokens.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let blank_hash = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2")?,
        artifact_ref("artifact-llm-prompt-blank")?,
    )
    .with_tool_schema_hash(" ");
    assert!(matches!(
        blank_hash.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn llm_activity_task_contract_requires_llm_activity_and_queue() -> Result<(), Box<dyn Error>> {
    let request = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2")?,
        artifact_ref("artifact-llm-prompt-task")?,
    );
    let valid_task = ActivityTask::new(
        ActivityId::new("activity-llm-plan-valid")?,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/llm-valid")?,
    )
    .with_timeout_ms(30_000);
    LlmActivityTask::new(valid_task, request.clone()).validate()?;

    let non_llm_type = ActivityTask::new(
        ActivityId::new("activity-llm-plan-type")?,
        ActivityType::new("tool.web")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/llm-type")?,
    );
    assert!(matches!(
        LlmActivityTask::new(non_llm_type, request.clone()).validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let non_llm_queue = ActivityTask::new(
        ActivityId::new("activity-llm-plan-queue")?,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("tool.web")?,
        IdempotencyKey::new("run/activity/llm-queue")?,
    );
    assert!(matches!(
        LlmActivityTask::new(non_llm_queue, request).validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}
