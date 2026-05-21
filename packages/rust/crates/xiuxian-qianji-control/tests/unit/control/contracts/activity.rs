use std::error::Error;

use xiuxian_qianji_control::{
    ActivityId, ActivityRetryPolicy, ActivityTask, ActivityType, ControlError, IdempotencyKey,
    TaskQueue,
};

#[test]
fn activity_task_contract_validates_retry_policy_and_timeout() -> Result<(), Box<dyn Error>> {
    let valid_task = ActivityTask::new(
        ActivityId::new("activity-valid")?,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/valid")?,
    )
    .with_retry_policy(
        ActivityRetryPolicy::new(3)?
            .with_initial_interval_ms(100)
            .with_max_interval_ms(1_000),
    )
    .with_timeout_ms(10_000);
    valid_task.validate()?;

    let zero_timeout = ActivityTask::new(
        ActivityId::new("activity-zero-timeout")?,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/zero-timeout")?,
    )
    .with_timeout_ms(0);
    assert!(matches!(
        zero_timeout.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let invalid_retry = ActivityRetryPolicy {
        max_attempts: 2,
        initial_interval_ms: 0,
        max_interval_ms: None,
        backoff_multiplier_millis: 0,
        non_retryable_error_codes: Vec::new(),
    };
    let invalid_task = ActivityTask::new(
        ActivityId::new("activity-invalid-retry")?,
        ActivityType::new("llm.plan")?,
        TaskQueue::new("llm.openai")?,
        IdempotencyKey::new("run/activity/invalid-retry")?,
    )
    .with_retry_policy(invalid_retry);
    assert!(matches!(
        invalid_task.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}
