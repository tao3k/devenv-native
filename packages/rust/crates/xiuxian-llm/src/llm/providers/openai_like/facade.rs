//! OpenAI-compatible provider construction and Responses API transport.

#[cfg(feature = "provider-litellm")]
use super::responses::{
    OpenAiResponsesAssistantOutput, build_openai_responses_payload, parse_openai_responses_stream,
    summarize_openai_responses_input, validate_openai_responses_input_tool_chain,
};
#[cfg(feature = "provider-litellm")]
use crate::llm::{Base64ImageSource, resolve_image_source_to_base64};
#[cfg(feature = "provider-litellm")]
use crate::llm::{HttpContentType, LlmError, LlmResult, sanitize_user_visible};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::openai_like::{OpenAILikeConfig, OpenAILikeProvider};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::chat::{
    ChatMessage as LiteChatMessage, ChatRequest as LiteChatRequest,
};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::content::{ContentPart, ImageSource};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::message::MessageContent;
#[cfg(feature = "provider-litellm")]
use reqwest::StatusCode;
#[cfg(feature = "provider-litellm")]
use reqwest::header::CONTENT_TYPE;
#[cfg(feature = "provider-litellm")]
use std::collections::HashMap;
#[cfg(feature = "provider-litellm")]
use std::time::Duration;
#[cfg(feature = "provider-litellm")]
use tokio::time::{sleep, timeout};
#[cfg(feature = "provider-litellm")]
use tracing::{debug, warn};

#[cfg(feature = "provider-litellm")]
const OPENAI_LIKE_STREAM_REQUIRED_HINT: &str = "stream must be set to true";
#[cfg(feature = "provider-litellm")]
const OPENAI_RESPONSES_MAX_ATTEMPTS: usize = 3;
#[cfg(feature = "provider-litellm")]
const OPENAI_RESPONSES_HEADER_TIMEOUT_SECS: u64 = 10;
#[cfg(feature = "provider-litellm")]
const OPENAI_RESPONSES_RETRY_BASE_DELAY_MS: u64 = 250;
#[cfg(feature = "provider-litellm")]
const OPENAI_RESPONSES_RETRY_MAX_DELAY_MS: u64 = 1_000;

#[cfg(feature = "provider-litellm")]
/// `litellm-rs` OpenAI-like provider handle for custom OpenAI-compatible bases.
pub type LiteLlmOpenAILikeProvider = OpenAILikeProvider;

#[cfg(feature = "provider-litellm")]
/// Build an OpenAI-compatible provider for custom base endpoints.
///
/// # Errors
///
/// Returns an error when provider initialization fails.
pub async fn build_openai_like_provider(
    api_base: String,
    api_key: Option<String>,
    timeout_secs: u64,
) -> LlmResult<LiteLlmOpenAILikeProvider> {
    let mut config = OpenAILikeConfig::new(api_base).with_timeout(timeout_secs);
    config.base.api_key = api_key;
    config.base.max_retries = 3;
    if config.base.api_key.is_none() {
        config.skip_api_key = true;
    }
    LiteLlmOpenAILikeProvider::new(config)
        .await
        .map_err(|error| LlmError::ProviderInitializationFailed {
            provider: "openai_like",
            reason: sanitize_user_visible(&error.to_string()),
        })
}

/// Inlines image URLs into base64 for OpenAI-compatible providers.
///
/// # Errors
///
/// Returns an error when remote image fetch or base64 conversion fails.
#[cfg(feature = "provider-litellm")]
pub async fn inline_openai_compatible_image_urls(
    client: &reqwest::Client,
    request: &LiteChatRequest,
) -> LlmResult<LiteChatRequest> {
    inline_openai_compatible_image_urls_impl(client, request).await
}

#[cfg(feature = "provider-litellm")]
async fn inline_openai_compatible_image_urls_impl(
    client: &reqwest::Client,
    request: &LiteChatRequest,
) -> LlmResult<LiteChatRequest> {
    let mut new_request = request.clone();
    let mut image_cache: HashMap<String, Base64ImageSource> = HashMap::new();

    for message in &mut new_request.messages {
        inline_message_images(client, &mut image_cache, message).await?;
    }

    Ok(new_request)
}

#[cfg(feature = "provider-litellm")]
async fn inline_message_images(
    client: &reqwest::Client,
    image_cache: &mut HashMap<String, Base64ImageSource>,
    message: &mut LiteChatMessage,
) -> LlmResult<()> {
    let Some(MessageContent::Parts(parts)) = message.content.clone() else {
        return Ok(());
    };

    let mut converted_parts = Vec::with_capacity(parts.len());
    for part in parts {
        converted_parts.push(inline_content_part_image(client, image_cache, part).await?);
    }
    message.content = Some(MessageContent::Parts(converted_parts));
    Ok(())
}

#[cfg(feature = "provider-litellm")]
async fn inline_content_part_image(
    client: &reqwest::Client,
    image_cache: &mut HashMap<String, Base64ImageSource>,
    part: ContentPart,
) -> LlmResult<ContentPart> {
    let ContentPart::ImageUrl { image_url } = part else {
        return Ok(part);
    };
    let image_ref = image_url.url;
    let source = cached_image_source(client, image_cache, &image_ref).await?;

    Ok(ContentPart::Image {
        source: ImageSource {
            media_type: source.media_type.to_string(),
            data: source.data,
        },
        detail: image_url.detail,
        image_url: None,
    })
}

#[cfg(feature = "provider-litellm")]
async fn cached_image_source(
    client: &reqwest::Client,
    image_cache: &mut HashMap<String, Base64ImageSource>,
    image_ref: &str,
) -> LlmResult<Base64ImageSource> {
    if let Some(cached) = image_cache.get(image_ref) {
        return Ok(cached.clone());
    }
    let resolved = resolve_image_source_to_base64(client, image_ref).await?;
    image_cache.insert(image_ref.to_string(), resolved.clone());
    Ok(resolved)
}

/// Detect the canonical OpenAI-compatible error indicating streaming is mandatory.
#[cfg(feature = "provider-litellm")]
#[must_use]
pub fn is_openai_like_stream_required_error_message(rendered: &str) -> bool {
    rendered
        .to_ascii_lowercase()
        .contains(OPENAI_LIKE_STREAM_REQUIRED_HINT)
}

/// Execute a custom-base `OpenAI` `/responses` request and parse assistant output.
///
/// # Errors
///
/// Returns an error when image inlining, HTTP transport, status validation, or stream parsing fails.
#[cfg(feature = "provider-litellm")]
pub async fn execute_openai_responses_request(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: Option<&str>,
    request: &LiteChatRequest,
) -> LlmResult<OpenAiResponsesAssistantOutput> {
    let request = inline_openai_compatible_image_urls(client, request).await?;
    let payload = build_openai_responses_payload(&request);
    let input_summary = summarize_openai_responses_input(&payload.payload);
    validate_openai_responses_input_tool_chain(&payload.payload)?;
    debug!(
        event = "xiuxian.llm.providers.openai_like.responses.dispatch",
        endpoint,
        input_summary = truncate_for_log(&input_summary, 1200),
        "dispatching OpenAI-compatible /responses request"
    );
    let max_attempts = OPENAI_RESPONSES_MAX_ATTEMPTS.max(1);
    let mut attempt = 1usize;

    loop {
        let response =
            match send_openai_responses_request_once(client, endpoint, api_key, &payload.payload)
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    if retry_openai_responses_attempt(endpoint, attempt, max_attempts, &error).await
                    {
                        attempt = attempt.saturating_add(1);
                        continue;
                    }
                    return Err(error);
                }
            };

        let (status, content_type, body) = read_openai_responses_http_body(response).await?;
        if !status.is_success() {
            let sanitized_reason = sanitize_user_visible(&body);
            if status.is_client_error()
                || sanitized_reason
                    .to_ascii_lowercase()
                    .contains("no tool call found for function call output")
            {
                warn!(
                    event = "xiuxian.llm.providers.openai_like.responses.failed",
                    endpoint,
                    status = %status,
                    reason_preview = truncate_for_log(&sanitized_reason, 240),
                    input_summary = truncate_for_log(&input_summary, 1200),
                    "OpenAI-compatible /responses request failed"
                );
            }
            let error = LlmError::RequestFailed {
                status,
                content_type: HttpContentType::new(content_type),
                reason: sanitized_reason.clone(),
            };
            if retry_openai_responses_attempt(endpoint, attempt, max_attempts, &error).await {
                attempt = attempt.saturating_add(1);
                continue;
            }
            return Err(error);
        }

        return parse_openai_responses_stream(&body, &payload.alias_to_original_tool_name).map_err(
            |error| match error {
                LlmError::Internal { message } => LlmError::Internal {
                    message: format!(
                        "responses stream parse failed: {message}; raw_body_preview={}",
                        truncate_for_log(&body, 400)
                    ),
                },
                other => other,
            },
        );
    }
}

#[cfg(feature = "provider-litellm")]
async fn send_openai_responses_request_once(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: Option<&str>,
    payload: &serde_json::Value,
) -> LlmResult<reqwest::Response> {
    let mut response = client
        .post(endpoint)
        .header(CONTENT_TYPE, "application/json")
        .json(payload);
    if let Some(key) = api_key {
        response = response.header("Authorization", format!("Bearer {key}"));
    }

    let header_timeout = Duration::from_secs(OPENAI_RESPONSES_HEADER_TIMEOUT_SECS);
    match timeout(header_timeout, response.send()).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(source)) => Err(LlmError::ConnectionFailed { source }),
        Err(_) => Err(LlmError::ResponseHeadersTimedOut {
            timeout_secs: header_timeout.as_secs(),
        }),
    }
}

#[cfg(feature = "provider-litellm")]
async fn read_openai_responses_http_body(
    response: reqwest::Response,
) -> LlmResult<(StatusCode, String, String)> {
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map_or_else(|| "application/octet-stream".to_string(), str::to_string);
    let body = response
        .text()
        .await
        .map_err(|source| LlmError::ResponseBodyReadFailed { source })?;
    Ok((status, content_type, body))
}

#[cfg(feature = "provider-litellm")]
async fn retry_openai_responses_attempt(
    endpoint: &str,
    attempt: usize,
    max_attempts: usize,
    error: &LlmError,
) -> bool {
    if !should_retry_openai_responses_error(error) || attempt >= max_attempts {
        return false;
    }

    let delay = retry_delay_for_openai_responses_attempt(attempt);
    match error {
        LlmError::ConnectionFailed { .. } => {
            warn!(
                event = "xiuxian.llm.providers.openai_like.responses.retry_network",
                endpoint,
                attempt,
                max_attempts,
                backoff_ms = delay.as_millis(),
                error = %error,
                "OpenAI-compatible /responses request hit transient transport failure; retrying"
            );
        }
        LlmError::ResponseHeadersTimedOut { timeout_secs } => {
            warn!(
                event = "xiuxian.llm.providers.openai_like.responses.retry_headers",
                endpoint,
                attempt,
                max_attempts,
                header_timeout_secs = timeout_secs,
                backoff_ms = delay.as_millis(),
                "OpenAI-compatible /responses request timed out before response headers; retrying"
            );
        }
        LlmError::RequestFailed { status, reason, .. } => {
            warn!(
                event = "xiuxian.llm.providers.openai_like.responses.retry_status",
                endpoint,
                attempt,
                max_attempts,
                status = %status,
                backoff_ms = delay.as_millis(),
                reason_preview = truncate_for_log(reason, 240),
                "OpenAI-compatible /responses request hit transient upstream status; retrying"
            );
        }
        _ => return false,
    }

    sleep(delay).await;
    true
}

#[cfg(feature = "provider-litellm")]
fn should_retry_openai_responses_error(error: &LlmError) -> bool {
    match error {
        LlmError::ConnectionFailed { source } => {
            let rendered = source.to_string().to_ascii_lowercase();
            source.is_connect()
                || source.is_timeout()
                || rendered.contains("error sending request for url")
                || rendered.contains("connection reset")
                || rendered.contains("connection termination")
                || rendered.contains("disconnect/reset before headers")
        }
        LlmError::ResponseHeadersTimedOut { .. } => true,
        LlmError::RequestFailed { status, reason, .. } => {
            matches!(
                *status,
                StatusCode::BAD_GATEWAY
                    | StatusCode::SERVICE_UNAVAILABLE
                    | StatusCode::GATEWAY_TIMEOUT
            ) || has_retryable_openai_responses_reason(reason)
        }
        _ => false,
    }
}

#[cfg(feature = "provider-litellm")]
fn has_retryable_openai_responses_reason(reason: &str) -> bool {
    let lower = reason.to_ascii_lowercase();
    lower.contains("upstream connect error")
        || lower.contains("disconnect/reset before headers")
        || lower.contains("connection termination")
        || lower.contains("upstream request timeout")
}

#[cfg(feature = "provider-litellm")]
fn retry_delay_for_openai_responses_attempt(attempt: usize) -> Duration {
    let shift = u32::try_from(attempt.saturating_sub(1).min(6)).unwrap_or(6);
    let factor = 1_u64.checked_shl(shift).unwrap_or(u64::MAX);
    let delay_ms = OPENAI_RESPONSES_RETRY_BASE_DELAY_MS.saturating_mul(factor);
    Duration::from_millis(delay_ms.min(OPENAI_RESPONSES_RETRY_MAX_DELAY_MS))
}

#[cfg(feature = "provider-litellm")]
fn truncate_for_log(raw: &str, limit: usize) -> String {
    if raw.chars().count() <= limit {
        return raw.to_string();
    }
    raw.chars().take(limit).collect::<String>()
}
