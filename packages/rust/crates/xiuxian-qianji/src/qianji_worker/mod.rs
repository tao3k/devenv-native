//! Shared Qianji activity worker execution core.
//!
//! CLI command parsing/rendering lives under `qianji_cli`. This module owns
//! provider-side execution facts that can be reused by qianji-server, tests,
//! and CLI adapters without importing CLI internals.

mod activity_result;
pub(crate) mod artifact;
mod openai_compatible;
mod worker_once;

pub(crate) use activity_result::{ActivityExecutorOutcome, control_error, invalid_input};
pub(crate) use openai_compatible::{
    OpenAiCompatibleLlmExecutionRequest, execute_openai_compatible_llm,
};
pub(crate) use worker_once::{
    OpenAiCompatibleWorkerOnceOutput, OpenAiCompatibleWorkerOnceRequest,
    run_openai_compatible_worker_once_for_run,
};
