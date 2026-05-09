//! Anthropic provider helpers and messages transport facade.

mod http;
mod media;
mod provider;
mod request;
mod response;
mod routing;
mod types;

pub use http::{
    AnthropicMessagesHttpRequest, send_anthropic_messages_json_with_retry,
    send_anthropic_messages_with_retry,
};
pub use media::normalize_anthropic_image_media_type;
#[cfg(feature = "provider-litellm")]
pub use provider::{LiteLlmAnthropicProvider, build_anthropic_provider};
#[cfg(feature = "provider-litellm")]
pub use request::{
    AnthropicMessagesExecutionRequest, build_anthropic_messages_body_from_litellm_request,
    build_anthropic_messages_body_from_litellm_request_with_image_hook,
    build_anthropic_messages_body_from_request,
    convert_litellm_messages_to_anthropic_with_image_hook,
    execute_anthropic_messages_from_litellm_request,
    execute_anthropic_messages_from_litellm_request_with_image_hook,
    split_anthropic_system_messages,
};
pub use response::parse_anthropic_messages_response;
pub use routing::{
    AnthropicTransportKeyResolution, ProviderApiKeyRef, anthropic_custom_base_transport_label,
    anthropic_custom_base_transport_order, anthropic_messages_endpoint_from_base,
    execute_anthropic_custom_base_fallback, is_anthropic_protocol_mismatch,
    is_official_anthropic_base, prefers_minimax_transport,
    resolve_custom_base_transport_api_key_from_values, should_bypass_anthropic_model_validation,
    summarize_anthropic_custom_base_failures,
};
pub use types::{
    AnthropicCustomBaseFallbackFailure, AnthropicCustomBaseTransport, AnthropicParsedResponse,
    AnthropicToolUse, DEFAULT_ANTHROPIC_KEY_ENV,
};
