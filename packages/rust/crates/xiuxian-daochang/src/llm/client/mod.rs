//! LLM client coordinates chat payloads, provider transport, retry policy, and tool-call normalization.

mod chat;
mod init;
mod test_api;
mod types;

pub(crate) use test_api::{
    enforce_tool_message_integrity_for_tests, test_resolve_backend_mode_for_inference_url,
};
pub use types::{LlmClient, LlmInFlightSnapshot};
