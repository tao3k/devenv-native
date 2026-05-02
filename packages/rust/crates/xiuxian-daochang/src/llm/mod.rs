//! LLM namespace: backend selection, request mapping, and chat client.

mod backend;
mod client;
mod compat;
#[cfg(feature = "agent-provider-litellm")]
mod converters;
mod protocol;
mod providers;
pub(crate) mod test_api;
mod tools;
mod types;

pub(crate) use backend::LlmBackendMode;
pub use client::{LlmClient, LlmInFlightSnapshot};
#[cfg(feature = "agent-provider-litellm")]
pub(crate) use converters::{
    chat_message_to_litellm_message, chat_message_to_litellm_message_for_openai_chat,
    content_from_litellm, tool_call_from_litellm,
};
pub(crate) use tools::PreparedTool;
pub use types::AssistantMessage;
