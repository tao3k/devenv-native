//! Anthropic endpoint, transport-order, and key-resolution helpers.

use std::fmt::Display;
use std::future::Future;

use reqwest::Url;

use super::types::{AnthropicCustomBaseFallbackFailure, AnthropicCustomBaseTransport};

const OFFICIAL_ANTHROPIC_HOST: &str = "api.anthropic.com";

/// Normalize an Anthropic-compatible base URL to a concrete `/v1/messages` endpoint.
#[must_use]
pub fn anthropic_messages_endpoint_from_base(api_base: &str) -> String {
    let trimmed = api_base.trim_end_matches('/');
    if trimmed.ends_with("/v1/messages") || trimmed.ends_with("/messages") {
        return trimmed.to_string();
    }
    if trimmed.ends_with("/v1") {
        return format!("{trimmed}/messages");
    }
    format!("{trimmed}/v1/messages")
}

/// Check whether a base URL points to the official Anthropic host.
#[must_use]
pub fn is_official_anthropic_base(api_base: &str) -> bool {
    Url::parse(api_base)
        .ok()
        .and_then(|url| {
            url.host_str()
                .map(|host| host.eq_ignore_ascii_case(OFFICIAL_ANTHROPIC_HOST))
        })
        .unwrap_or(false)
}

/// Check whether Anthropic model validation should be bypassed.
#[must_use]
pub fn should_bypass_anthropic_model_validation(api_base: &str) -> bool {
    !is_official_anthropic_base(api_base)
}

/// Check whether a model name should prefer `MiniMax` transport fallback.
#[must_use]
pub fn prefers_minimax_transport(model: &str) -> bool {
    let lower = model.trim().to_ascii_lowercase();
    lower.starts_with("glm-") || lower.starts_with("minimax-") || lower.starts_with("minimax/")
}

/// Determine custom-base transport order for Anthropic provider mode.
#[must_use]
pub fn anthropic_custom_base_transport_order(model: &str) -> [AnthropicCustomBaseTransport; 3] {
    if prefers_minimax_transport(model) {
        [
            AnthropicCustomBaseTransport::Minimax,
            AnthropicCustomBaseTransport::OpenAi,
            AnthropicCustomBaseTransport::AnthropicMessagesBypass,
        ]
    } else {
        [
            AnthropicCustomBaseTransport::OpenAi,
            AnthropicCustomBaseTransport::Minimax,
            AnthropicCustomBaseTransport::AnthropicMessagesBypass,
        ]
    }
}

/// Render transport label for telemetry/logging.
#[must_use]
pub const fn anthropic_custom_base_transport_label(
    transport: AnthropicCustomBaseTransport,
) -> &'static str {
    match transport {
        AnthropicCustomBaseTransport::OpenAi => "openai",
        AnthropicCustomBaseTransport::Minimax => "minimax",
        AnthropicCustomBaseTransport::AnthropicMessagesBypass => "anthropic_messages_bypass",
    }
}

/// Resolve transport-specific API key precedence for anthropic custom-base fallback.
#[derive(Debug, Clone, Copy)]
pub struct AnthropicTransportKeyResolution<'a> {
    /// Selected custom-base transport.
    pub transport: AnthropicCustomBaseTransport,
    /// Explicit request API key.
    pub explicit_api_key: Option<ProviderApiKeyRef<'a>>,
    /// Configured custom-base key.
    pub configured_key: Option<ProviderApiKeyRef<'a>>,
    /// OpenAI-compatible fallback key.
    pub openai_key: Option<ProviderApiKeyRef<'a>>,
    /// Minimax fallback key.
    pub minimax_key: Option<ProviderApiKeyRef<'a>>,
    /// Anthropic fallback key.
    pub anthropic_key: Option<ProviderApiKeyRef<'a>>,
}

/// Borrowed provider API key candidate.
#[derive(Debug, Clone, Copy)]
pub struct ProviderApiKeyRef<'a>(&'a str);

impl<'a> ProviderApiKeyRef<'a> {
    /// Creates a borrowed API key candidate.
    #[must_use]
    pub const fn new(value: &'a str) -> Self {
        Self(value)
    }

    /// Returns the raw API key candidate.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Resolve transport-specific API key precedence for anthropic custom-base fallback.
#[must_use]
pub fn resolve_custom_base_transport_api_key_from_values(
    request: AnthropicTransportKeyResolution<'_>,
) -> Option<String> {
    let explicit = normalize_optional_key_ref(request.explicit_api_key);
    if explicit.is_some() {
        return explicit;
    }

    let configured = normalize_optional_key_ref(request.configured_key);
    let openai = normalize_optional_key_ref(request.openai_key);
    let minimax = normalize_optional_key_ref(request.minimax_key);
    let anthropic = normalize_optional_key_ref(request.anthropic_key);

    match request.transport {
        AnthropicCustomBaseTransport::OpenAi => {
            first_present_key(&[openai, configured, minimax, anthropic])
        }
        AnthropicCustomBaseTransport::Minimax => {
            first_present_key(&[minimax, openai, configured, anthropic])
        }
        AnthropicCustomBaseTransport::AnthropicMessagesBypass => {
            first_present_key(&[configured, anthropic, openai, minimax])
        }
    }
}

/// Render failed custom-base fallback attempts into a stable summary string.
#[must_use]
pub fn summarize_anthropic_custom_base_failures<E: Display>(
    attempts: &[(AnthropicCustomBaseTransport, E)],
) -> String {
    let mut parts = Vec::with_capacity(attempts.len());
    for (transport, error) in attempts {
        parts.push(format!(
            "{}: {}",
            anthropic_custom_base_transport_label(*transport),
            error
        ));
    }
    parts.join(" | ")
}

/// Execute Anthropic custom-base fallback attempts in canonical transport order.
///
/// # Errors
///
/// Returns [`AnthropicCustomBaseFallbackFailure`] when all transport attempts fail.
pub async fn execute_anthropic_custom_base_fallback<T, E, F, Fut>(
    model: &str,
    mut attempt_transport: F,
) -> Result<T, AnthropicCustomBaseFallbackFailure<E>>
where
    F: FnMut(AnthropicCustomBaseTransport) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let mut failures = Vec::with_capacity(3);
    for transport in anthropic_custom_base_transport_order(model) {
        match attempt_transport(transport).await {
            Ok(value) => return Ok(value),
            Err(error) => failures.push((transport, error)),
        }
    }
    Err(AnthropicCustomBaseFallbackFailure { attempts: failures })
}

fn normalize_optional_key(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_optional_key_ref(raw: Option<ProviderApiKeyRef<'_>>) -> Option<String> {
    normalize_optional_key(raw.map(ProviderApiKeyRef::as_str))
}

fn first_present_key(candidates: &[Option<String>]) -> Option<String> {
    candidates.iter().find_map(std::clone::Clone::clone)
}

/// Check whether an Anthropic custom-base error indicates protocol mismatch.
#[must_use]
pub fn is_anthropic_protocol_mismatch(error_text: &str) -> bool {
    let lower = error_text.to_ascii_lowercase();
    if !(lower.contains("http 400") || lower.contains("bad request")) {
        return false;
    }

    lower.contains("messages 参数非法")
        || lower.contains("messages parameter")
        || lower.contains("messages param")
        || lower.contains("invalid messages")
}
