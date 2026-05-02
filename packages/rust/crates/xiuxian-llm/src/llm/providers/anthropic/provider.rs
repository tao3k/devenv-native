//! Anthropic `LiteLLM` provider construction.

#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};

#[cfg(feature = "provider-litellm")]
use crate::llm::error::sanitize_user_visible;
#[cfg(feature = "provider-litellm")]
use crate::llm::{LlmError, LlmResult};

#[cfg(feature = "provider-litellm")]
/// `litellm-rs` Anthropic provider handle used by callers.
pub type LiteLlmAnthropicProvider = AnthropicProvider;

#[cfg(feature = "provider-litellm")]
/// Build an Anthropic provider with runtime overrides.
///
/// # Errors
///
/// Returns an error when provider initialization fails (invalid configuration,
/// unsupported endpoint shape, or client construction failure).
pub async fn build_anthropic_provider(
    api_base: String,
    api_key: String,
    timeout_secs: u64,
) -> LlmResult<LiteLlmAnthropicProvider> {
    let mut config = AnthropicConfig::new(api_key);
    config.base_url = api_base;
    config.request_timeout = timeout_secs;
    config.connect_timeout = timeout_secs.clamp(5, 60);

    LiteLlmAnthropicProvider::new(config).map_err(|error| LlmError::ProviderInitializationFailed {
        provider: "anthropic",
        reason: sanitize_user_visible(&error.to_string()),
    })
}
