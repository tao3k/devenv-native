use std::io;

use serde_json::Value;
use xiuxian_qianji_control::ErrorCode;

use crate::qianji_worker::{ActivityExecutorOutcome, invalid_input};

pub(super) fn provider_failure(
    error_code: &'static str,
    message: impl Into<String>,
    retryable: bool,
    metadata: Value,
) -> io::Result<ActivityExecutorOutcome> {
    Ok(ActivityExecutorOutcome::Fail {
        error_code: ErrorCode::new(error_code)
            .map_err(|error| invalid_input(format!("{error}")))?,
        message: message.into(),
        retryable,
        metadata,
    })
}
