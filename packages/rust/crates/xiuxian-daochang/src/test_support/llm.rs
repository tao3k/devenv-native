pub use crate::llm::test_api::{
    ChatCompletionRequest, DEFAULT_ANTHROPIC_KEY_ENV, DEFAULT_MINIMAX_KEY_ENV,
    DEFAULT_OPENAI_KEY_ENV, LiteLlmProviderMode, LiteLlmWireApi, LlmBackendMode, ProviderSettings,
    ToolMessageIntegrityReport, chat_completion_request_to_value, enforce_tool_message_integrity,
    extract_api_base_from_inference_url, is_openai_like_stream_required_error, parse_backend_mode,
    parse_tools_json, resolve_backend_mode_for_inference_url, resolve_provider_settings_with_env,
    should_use_openai_like_for_base,
};

#[cfg(feature = "agent-provider-litellm")]
pub use crate::llm::test_api::{
    CustomBaseFallbackTransport, build_responses_payload_from_chat_completion_request,
    chat_message_to_litellm_message, chat_message_to_litellm_message_for_openai_chat,
    parse_responses_stream_tool_names, resolve_custom_base_transport_api_key_from_values,
};
