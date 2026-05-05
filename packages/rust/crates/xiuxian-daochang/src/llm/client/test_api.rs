//! Test adapter API for LLM client internals.

use crate::config::RuntimeSettings;
use crate::llm::LlmBackendMode;
use crate::session::ChatMessage;

pub(crate) fn enforce_tool_message_integrity_for_tests(
    messages: Vec<ChatMessage>,
) -> (
    Vec<ChatMessage>,
    super::super::protocol::ToolMessageIntegrityReport,
) {
    super::super::protocol::enforce_tool_message_integrity(messages)
}

pub(crate) fn test_resolve_backend_mode_for_inference_url(
    runtime_settings: &RuntimeSettings,
    inference_url: &str,
    env_backend_raw: Option<&str>,
) -> (LlmBackendMode, &'static str) {
    super::init::test_resolve_backend_mode_for_inference_url(
        runtime_settings,
        inference_url,
        env_backend_raw,
    )
}
