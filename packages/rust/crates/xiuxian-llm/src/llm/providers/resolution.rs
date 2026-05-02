//! Provider environment and runtime override resolution helpers.

use crate::llm::error::LlmError;

/// Resolve provider API key from explicit input or environment fallback chain.
///
/// Resolution order:
/// 1. `explicit_api_key` when non-empty.
/// 2. environment variable `primary_env`.
/// 3. environment variable `fallback_env`.
#[must_use]
pub fn resolve_api_key_with_env(
    explicit_api_key: Option<&str>,
    primary_env: &str,
    fallback_env: &str,
) -> Option<String> {
    explicit_api_key
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .or_else(|| read_non_empty_env(primary_env))
        .or_else(|| read_non_empty_env(fallback_env))
}

fn read_non_empty_env(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Parse a positive `usize` from optional raw text with fallback default.
#[must_use]
pub fn parse_positive_usize(raw: Option<&str>, default: usize) -> usize {
    let fallback = default.max(1);
    raw.and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(fallback)
}

/// Resolve a positive `usize` from an environment variable with fallback default.
#[must_use]
pub fn resolve_positive_usize_env(name: &str, default: usize) -> usize {
    let value = std::env::var(name).ok();
    parse_positive_usize(value.as_deref(), default)
}

/// Resolve required provider API key from explicit input or environment fallback chain.
///
/// # Errors
///
/// Returns `LlmError::ProviderInitializationFailed` when all sources are missing/empty.
pub fn resolve_required_api_key_with_env(
    explicit_api_key: Option<&str>,
    primary_env: &str,
    fallback_env: &str,
    provider: &'static str,
) -> Result<String, LlmError> {
    resolve_api_key_with_env(explicit_api_key, primary_env, fallback_env).ok_or_else(|| {
        LlmError::ProviderInitializationFailed {
            provider,
            reason: format!("missing {provider} api key; set {primary_env} or {fallback_env}"),
        }
    })
}

/// Normalize optional API base override from raw user/config input.
#[must_use]
pub fn normalize_optional_base_override(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(normalize_openai_compatible_base)
}

fn normalize_openai_compatible_base(raw: &str) -> String {
    let trimmed = raw.trim().trim_end_matches('/');
    let without_chat_suffix = trimmed
        .strip_suffix("/chat/completions")
        .unwrap_or(trimmed)
        .trim_end_matches('/');
    if without_chat_suffix.ends_with("/v1") {
        without_chat_suffix.to_string()
    } else {
        format!("{without_chat_suffix}/v1")
    }
}
