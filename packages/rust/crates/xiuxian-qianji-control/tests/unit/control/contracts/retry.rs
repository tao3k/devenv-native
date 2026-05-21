use std::error::Error;

use xiuxian_qianji_control::{
    ActivityFailure, ActivityRetryDecision, ActivityRetryPolicy, ActivityRetryStopReason,
    ControlError, ErrorCode,
};

#[test]
fn activity_retry_policy_contract_rejects_invalid_values() -> Result<(), Box<dyn Error>> {
    assert!(matches!(
        ActivityRetryPolicy::new(0),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    assert!(matches!(
        ActivityRetryPolicy::new(2)?.with_backoff_multiplier_millis(0),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    let inverted_interval = ActivityRetryPolicy::new(2)?
        .with_initial_interval_ms(1_000)
        .with_max_interval_ms(500);
    assert!(matches!(
        inverted_interval.validate(),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

#[test]
fn activity_retry_decision_allows_next_attempt_with_capped_backoff() -> Result<(), Box<dyn Error>> {
    let policy = ActivityRetryPolicy::new(3)?
        .with_initial_interval_ms(100)
        .with_max_interval_ms(150);

    assert_eq!(
        policy.decide_after_failure(&activity_failure("rate_limited", true, 1)?)?,
        ActivityRetryDecision::Retry {
            next_attempt: 2,
            backoff_ms: 100,
        }
    );
    assert_eq!(
        policy.decide_after_failure(&activity_failure("rate_limited", true, 2)?)?,
        ActivityRetryDecision::Retry {
            next_attempt: 3,
            backoff_ms: 150,
        }
    );

    Ok(())
}

#[test]
fn activity_retry_decision_denies_non_retryable_and_exhausted_failures()
-> Result<(), Box<dyn Error>> {
    let policy = ActivityRetryPolicy::new(2)?
        .with_non_retryable_error_code(ErrorCode::new("schema_invalid")?);

    assert_eq!(
        policy.decide_after_failure(&activity_failure("rate_limited", false, 1)?)?,
        ActivityRetryDecision::DoNotRetry {
            reason: ActivityRetryStopReason::FailureMarkedNonRetryable,
        }
    );
    assert_eq!(
        policy.decide_after_failure(&activity_failure("schema_invalid", true, 1)?)?,
        ActivityRetryDecision::DoNotRetry {
            reason: ActivityRetryStopReason::NonRetryableErrorCode,
        }
    );
    assert_eq!(
        policy.decide_after_failure(&activity_failure("rate_limited", true, 2)?)?,
        ActivityRetryDecision::DoNotRetry {
            reason: ActivityRetryStopReason::AttemptsExhausted,
        }
    );

    Ok(())
}

#[test]
fn activity_retry_decision_rejects_zero_attempt_failures() -> Result<(), Box<dyn Error>> {
    let policy = ActivityRetryPolicy::new(2)?;

    assert!(matches!(
        policy.decide_after_failure(&activity_failure("rate_limited", true, 0)?),
        Err(ControlError::InvalidEventSequence { .. })
    ));
    assert!(matches!(
        policy.retry_backoff_ms_after_failed_attempt(0),
        Err(ControlError::InvalidEventSequence { .. })
    ));

    Ok(())
}

fn activity_failure(
    error_code: &str,
    retryable: bool,
    attempt: u32,
) -> Result<ActivityFailure, Box<dyn Error>> {
    Ok(ActivityFailure {
        error_code: ErrorCode::new(error_code)?,
        message: format!("{error_code} failure"),
        retryable,
        attempt,
        metadata: serde_json::Value::Null,
    })
}
