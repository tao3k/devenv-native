//! LiteLLM provider builders shared across crates.

mod anthropic;
mod minimax;
mod openai;
mod openai_like;
mod resolution;

pub use anthropic::DEFAULT_ANTHROPIC_KEY_ENV;
#[cfg(feature = "provider-litellm")]
pub use anthropic::build_anthropic_messages_body_from_request;
#[cfg(feature = "provider-litellm")]
pub use anthropic::split_anthropic_system_messages;
pub use anthropic::{
    AnthropicCustomBaseFallbackFailure, AnthropicCustomBaseTransport, AnthropicMessagesHttpRequest,
    AnthropicParsedResponse, AnthropicToolUse, AnthropicTransportKeyResolution, ProviderApiKeyRef,
    anthropic_custom_base_transport_label, anthropic_custom_base_transport_order,
    anthropic_messages_endpoint_from_base, execute_anthropic_custom_base_fallback,
    is_anthropic_protocol_mismatch, is_official_anthropic_base,
    normalize_anthropic_image_media_type, parse_anthropic_messages_response,
    prefers_minimax_transport, resolve_custom_base_transport_api_key_from_values,
    send_anthropic_messages_json_with_retry, send_anthropic_messages_with_retry,
    should_bypass_anthropic_model_validation, summarize_anthropic_custom_base_failures,
};
#[cfg(feature = "provider-litellm")]
pub use anthropic::{
    AnthropicMessagesExecutionRequest, LiteLlmAnthropicProvider,
    build_anthropic_messages_body_from_litellm_request,
    build_anthropic_messages_body_from_litellm_request_with_image_hook, build_anthropic_provider,
    convert_litellm_messages_to_anthropic_with_image_hook,
    execute_anthropic_messages_from_litellm_request,
    execute_anthropic_messages_from_litellm_request_with_image_hook,
};
pub use minimax::DEFAULT_MINIMAX_KEY_ENV;
#[cfg(feature = "provider-litellm")]
pub use minimax::{LiteLlmMinimaxProvider, build_minimax_provider};
pub use openai::DEFAULT_OPENAI_KEY_ENV;
pub use openai::should_use_openai_like_for_base;
#[cfg(feature = "provider-litellm")]
pub use openai::{LiteLlmOpenAIProvider, build_openai_provider};
#[cfg(feature = "provider-litellm")]
pub use openai_like::{
    LiteLlmOpenAILikeProvider, OpenAiResponsesAssistantOutput, OpenAiResponsesPayload,
    OpenAiResponsesToolCall, OpenAiResponsesToolType, build_openai_like_provider,
    build_openai_responses_payload, execute_openai_responses_request,
    inline_openai_compatible_image_urls, is_openai_like_stream_required_error_message,
    parse_openai_responses_stream, remap_openai_responses_tool_name,
};
pub use resolution::{
    normalize_optional_base_override, parse_positive_usize, resolve_api_key_with_env,
    resolve_positive_usize_env, resolve_required_api_key_with_env,
};
