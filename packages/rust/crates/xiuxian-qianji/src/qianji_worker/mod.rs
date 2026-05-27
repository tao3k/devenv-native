//! Shared Qianji activity worker execution core.
//!
//! CLI command parsing/rendering lives under `qianji_cli`. This module owns
//! provider-side execution facts that can be reused by qianji-server, tests,
//! and CLI adapters without importing CLI internals.

pub(crate) mod artifact;
mod openai_compatible;
mod worker_once;

pub(crate) use openai_compatible::{
    OpenAiCompatibleLlmExecutionRequest, execute_openai_compatible_llm,
};
pub(crate) use worker_once::{
    OpenAiCompatibleWorkerOnceOutput, OpenAiCompatibleWorkerOnceRequest,
    run_openai_compatible_worker_once_for_run,
};

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
