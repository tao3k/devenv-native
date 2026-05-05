//! `MiniMax` `LiteLLM` provider construction.

#[cfg(feature = "provider-litellm")]
use crate::llm::error::sanitize_user_visible;
#[cfg(feature = "provider-litellm")]
use crate::llm::error::{LlmError, LlmResult};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::openai_like::{OpenAILikeConfig, OpenAILikeProvider};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::registry::catalog::get_definition;

#[cfg(feature = "provider-litellm")]
/// `litellm-rs` OpenAI-like provider handle used for `MiniMax` routing.
pub type LiteLlmMinimaxProvider = OpenAILikeProvider;

/// Default environment variable used to resolve `MiniMax` credentials.
pub const DEFAULT_MINIMAX_KEY_ENV: &str = "MINIMAX_API_KEY";

#[cfg(feature = "provider-litellm")]
/// Build a `MiniMax` provider from `litellm-rs` registry metadata.
///
/// # Errors
///
/// Returns an error when the `minimax` registry definition is unavailable or
/// when provider construction fails.
pub async fn build_minimax_provider(
    api_base_override: Option<String>,
    api_key: String,
    timeout_secs: u64,
) -> LlmResult<LiteLlmMinimaxProvider> {
    let definition =
        get_definition("minimax").ok_or_else(|| LlmError::ProviderRegistryMissing {
            provider: "minimax",
            reason: "litellm-rs registry missing minimax definition".to_string(),
        })?;
    let mut config: OpenAILikeConfig = definition
        .to_openai_like_config(Some(api_key.as_str()), api_base_override.as_deref())
        .with_timeout(timeout_secs);
    config.base.max_retries = 3;

    LiteLlmMinimaxProvider::new(config).await.map_err(|error| {
        LlmError::ProviderInitializationFailed {
            provider: "minimax",
            reason: sanitize_user_visible(&error.to_string()),
        }
    })
}
