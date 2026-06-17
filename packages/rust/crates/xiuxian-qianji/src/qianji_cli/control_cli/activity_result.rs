use crate::qianji_cli::invalid_input;

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

pub(crate) fn control_error(error: &xiuxian_qianji_control::ControlError) -> std::io::Error {
    invalid_input(format!("{error}"))
}
