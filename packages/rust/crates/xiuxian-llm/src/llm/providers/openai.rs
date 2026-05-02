//! `OpenAI` `LiteLLM` provider construction and endpoint routing.

#[cfg(feature = "provider-litellm")]
use crate::llm::error::sanitize_user_visible;
#[cfg(feature = "provider-litellm")]
use crate::llm::error::{LlmError, LlmResult};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::base::BaseConfig;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::openai::OpenAIProvider;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::openai::config::{OpenAIConfig, OpenAIFeatures};
#[cfg(feature = "provider-litellm")]
use std::collections::HashMap;

#[cfg(feature = "provider-litellm")]
/// `litellm-rs` `OpenAI` provider handle used by callers.
pub type LiteLlmOpenAIProvider = OpenAIProvider;

/// Default environment variable used to resolve `OpenAI` credentials.
pub const DEFAULT_OPENAI_KEY_ENV: &str = "OPENAI_API_KEY";

/// Returns `true` when a configured `OpenAI` base should be treated as a
/// generic `OpenAI`-compatible endpoint instead of official `OpenAI` transport.
///
/// Official `OpenAI` transport enforces `OpenAI` key prefix checks (`sk-`/`proj-`),
/// which is invalid for many proxy gateways that still speak `OpenAI` protocol.
#[must_use]
pub fn should_use_openai_like_for_base(api_base: &str) -> bool {
    let trimmed = api_base.trim();
    if trimmed.is_empty() {
        return false;
    }

    let Ok(parsed) = reqwest::Url::parse(trimmed) else {
        return true;
    };
    let Some(host) = parsed.host_str() else {
        return true;
    };

    !host.eq_ignore_ascii_case("api.openai.com")
}

#[cfg(feature = "provider-litellm")]
/// Build an `OpenAI` provider with runtime overrides.
///
/// # Errors
///
/// Returns an error when provider initialization fails (invalid configuration
/// or HTTP client setup failure).
pub async fn build_openai_provider(
    api_base: String,
    api_key: Option<String>,
    timeout_secs: u64,
) -> LlmResult<LiteLlmOpenAIProvider> {
    let config = OpenAIConfig {
        base: BaseConfig {
            api_key,
            api_base: Some(api_base),
            timeout: timeout_secs,
            max_retries: 3,
            headers: HashMap::default(),
            organization: None,
            api_version: None,
        },
        organization: None,
        project: None,
        model_mappings: HashMap::default(),
        features: OpenAIFeatures::default(),
    };

    LiteLlmOpenAIProvider::new(config).await.map_err(|error| {
        LlmError::ProviderInitializationFailed {
            provider: "openai",
            reason: sanitize_user_visible(&error.to_string()),
        }
    })
}
