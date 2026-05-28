#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ActivityExecutorOutcome {
    Complete {
        result: xiuxian_qianji_control::ActivityResult,
    },
    Fail {
        error_code: xiuxian_qianji_control::ErrorCode,
        message: String,
        retryable: bool,
        metadata: serde_json::Value,
    },
}

pub(crate) fn invalid_input(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into())
}

pub(crate) fn control_error(error: &xiuxian_qianji_control::ControlError) -> std::io::Error {
    invalid_input(format!("{error}"))
}
