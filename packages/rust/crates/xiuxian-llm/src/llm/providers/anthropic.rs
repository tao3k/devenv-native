use std::fmt::Display;
use std::future::Future;
#[cfg(feature = "provider-litellm")]
use std::future::ready;
use std::time::Duration;

use base64::Engine as _;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::chat::ChatMessage as LiteChatMessage;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::chat::ChatRequest as LiteChatRequest;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::content::ContentPart as LiteContentPart;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::message::{
    MessageContent as LiteMessageContent, MessageRole as LiteMessageRole,
};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
use reqwest::Url;
use serde_json::{Value, json};
use tokio::time::sleep;
use tracing::warn;

use crate::llm::error::sanitize_user_visible;
use crate::llm::error::{LlmError, LlmResult};
#[cfg(feature = "provider-litellm")]
use crate::llm::multimodal::{Base64ImageSource, resolve_image_source_to_base64};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::providers::anthropic::{AnthropicConfig, AnthropicProvider};

#[cfg(feature = "provider-litellm")]
/// `litellm-rs` Anthropic provider handle used by callers.
pub type LiteLlmAnthropicProvider = AnthropicProvider;

/// Default environment variable used to resolve Anthropic credentials.
pub const DEFAULT_ANTHROPIC_KEY_ENV: &str = "ANTHROPIC_API_KEY";
const OFFICIAL_ANTHROPIC_HOST: &str = "api.anthropic.com";

/// Transport order entry for Anthropic custom-base fallback orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicCustomBaseTransport {
    /// OpenAI-compatible provider transport.
    OpenAi,
    /// `MiniMax` OpenAI-like transport.
    Minimax,
    /// Direct Anthropic `/v1/messages` bypass transport.
    AnthropicMessagesBypass,
}

/// Failed Anthropic custom-base fallback attempts.
#[derive(Debug)]
pub struct AnthropicCustomBaseFallbackFailure<E> {
    attempts: Vec<(AnthropicCustomBaseTransport, E)>,
}

impl<E> AnthropicCustomBaseFallbackFailure<E> {
    /// Access failed transport attempts in execution order.
    #[must_use]
    pub fn attempts(&self) -> &[(AnthropicCustomBaseTransport, E)] {
        self.attempts.as_slice()
    }

    /// Return last failure payload (if any).
    #[must_use]
    pub fn last_error(&self) -> Option<&E> {
        self.attempts.last().map(|(_transport, error)| error)
    }

    /// Consume and return all failed attempts.
    #[must_use]
    pub fn into_attempts(self) -> Vec<(AnthropicCustomBaseTransport, E)> {
        self.attempts
    }
}

/// Structured `tool_use` item decoded from Anthropic `messages` response.
#[derive(Debug, Clone, PartialEq)]
pub struct AnthropicToolUse {
    /// Tool call identifier.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool input payload.
    pub input: Value,
}

/// Parsed subset of Anthropic `messages` response content used by callers.
#[derive(Debug, Clone, PartialEq)]
pub struct AnthropicParsedResponse {
    /// Concatenated plain text segments.
    pub text: Option<String>,
    /// Structured tool calls.
    pub tool_uses: Vec<AnthropicToolUse>,
}

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
#[must_use]
pub fn resolve_custom_base_transport_api_key_from_values(
    transport: AnthropicCustomBaseTransport,
    explicit_api_key: Option<&str>,
    configured_key: Option<&str>,
    openai_key: Option<&str>,
    minimax_key: Option<&str>,
    anthropic_key: Option<&str>,
) -> Option<String> {
    let explicit = normalize_optional_key(explicit_api_key);
    if explicit.is_some() {
        return explicit;
    }

    let configured = normalize_optional_key(configured_key);
    let openai = normalize_optional_key(openai_key);
    let minimax = normalize_optional_key(minimax_key);
    let anthropic = normalize_optional_key(anthropic_key);

    match transport {
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

/// Normalize image media type for Anthropic image blocks.
///
/// Anthropic `messages` image content only accepts a strict image MIME subset.
/// Unknown/opaque values (for example `application/octet-stream`) are normalized
/// by probing the base64 payload header.
#[must_use]
pub fn normalize_anthropic_image_media_type(media_type: &str, base64_data: &str) -> String {
    if let Some(normalized) = normalize_explicit_image_media_type(media_type) {
        return normalized.to_string();
    }
    if let Some(detected) = detect_image_media_type_from_base64(base64_data) {
        return detected.to_string();
    }
    "image/jpeg".to_string()
}

fn normalize_explicit_image_media_type(media_type: &str) -> Option<&'static str> {
    match media_type.trim().to_ascii_lowercase().as_str() {
        "image/png" => Some("image/png"),
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/webp" => Some("image/webp"),
        "image/gif" => Some("image/gif"),
        _ => None,
    }
}

fn detect_image_media_type_from_base64(base64_data: &str) -> Option<&'static str> {
    let payload = extract_base64_payload(base64_data);
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload)
        .ok()?;
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    None
}

fn extract_base64_payload(raw: &str) -> &str {
    let trimmed = raw.trim();
    if let Some((prefix, payload)) = trimmed.split_once(',')
        && prefix.to_ascii_lowercase().contains("base64")
    {
        return payload;
    }
    trimmed
}

/// Send Anthropic-compatible `messages` request with retry on transient transport errors.
///
/// # Errors
///
/// Returns `LlmError::Internal` when all retry attempts fail due to network transport issues.
/// Returns `LlmError::ConnectionFailed` when request building/sending fails on a non-retryable error.
pub async fn send_anthropic_messages_with_retry(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    body: &Value,
    attempts: usize,
) -> LlmResult<reqwest::Response> {
    let max_attempts = attempts.max(1);
    let mut attempt = 1usize;
    loop {
        let result = client
            .post(endpoint)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await;

        match result {
            Ok(response) => return Ok(response),
            Err(error) => {
                let retryable = is_retryable_network_error(&error);
                if !retryable || attempt >= max_attempts {
                    if retryable {
                        return Err(LlmError::Internal {
                            message: format!(
                                "anthropic request network error after {attempt}/{max_attempts} attempt(s): {error}"
                            ),
                        });
                    }
                    return Err(LlmError::ConnectionFailed { source: error });
                }
                let backoff = retry_backoff_for_attempt(attempt);
                warn!(
                    event = "xiuxian.llm.providers.anthropic_http.network_retry",
                    endpoint,
                    attempt,
                    max_attempts,
                    backoff_ms = backoff.as_millis(),
                    error = %error,
                    "Anthropic request hit transient network error; retrying"
                );
                sleep(backoff).await;
                attempt = attempt.saturating_add(1);
            }
        }
    }
}

/// Send Anthropic-compatible `messages` request and decode JSON body.
///
/// # Errors
///
/// Returns `LlmError::Internal` when the endpoint returns a non-success status or
/// when the response payload cannot be decoded as JSON.
pub async fn send_anthropic_messages_json_with_retry(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    body: &Value,
    attempts: usize,
) -> LlmResult<Value> {
    let response =
        send_anthropic_messages_with_retry(client, endpoint, api_key, body, attempts).await?;
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.map_err(|source| LlmError::Internal {
            message: format!(
                "anthropic response read failed after HTTP {status}: {}",
                sanitize_user_visible(&source.to_string())
            ),
        })?;
        return Err(LlmError::Internal {
            message: format!(
                "anthropic request failed with HTTP {status}: {}",
                sanitize_user_visible(&error_text)
            ),
        });
    }

    response.json().await.map_err(|source| LlmError::Internal {
        message: format!(
            "anthropic response json decode failed: {}",
            sanitize_user_visible(&source.to_string())
        ),
    })
}

fn retry_backoff_for_attempt(attempt: usize) -> Duration {
    match attempt {
        1 => Duration::from_millis(250),
        2 => Duration::from_millis(500),
        _ => Duration::from_secs(1),
    }
}

fn is_retryable_network_error(error: &reqwest::Error) -> bool {
    if error.is_connect() || error.is_timeout() {
        return true;
    }
    let text = error.to_string().to_ascii_lowercase();
    text.contains("error sending request")
        || text.contains("connection reset")
        || text.contains("connection aborted")
        || text.contains("connection closed")
        || text.contains("broken pipe")
}

/// Build Anthropic `messages` request body from a `litellm-rs` chat request.
#[cfg(feature = "provider-litellm")]
#[must_use]
pub fn build_anthropic_messages_body_from_request(
    request: &LiteChatRequest,
    messages: &[Value],
    system_message: Option<String>,
) -> Value {
    let normalized_messages = normalize_anthropic_messages(messages);
    let mut body = json!({
        "model": request.model,
        "max_tokens": request.max_tokens.unwrap_or(4096),
        "messages": normalized_messages,
    });

    if let Some(system) = system_message {
        body["system"] = json!(system);
    }
    if let Some(temperature) = request.temperature {
        body["temperature"] = json!(temperature);
    }
    if let Some(top_p) = request.top_p {
        body["top_p"] = json!(top_p);
    }
    if let Some(stop) = &request.stop {
        body["stop_sequences"] = json!(stop);
    }
    if let Some(tools) = &request.tools {
        body["tools"] = json!(convert_litellm_tools_to_anthropic(tools));
        if let Some(tool_choice) = &request.tool_choice {
            body["tool_choice"] = convert_litellm_tool_choice_to_anthropic(tool_choice);
        }
    }

    body
}

/// Build Anthropic `messages` body by converting a full `litellm-rs` chat request.
///
/// This helper performs system message extraction and multimodal message conversion,
/// then applies Anthropic body shaping (`model/max_tokens/tools/temperature`).
///
/// # Errors
///
/// Returns an error when image URL parts cannot be resolved into base64 payloads.
#[cfg(feature = "provider-litellm")]
pub async fn build_anthropic_messages_body_from_litellm_request(
    client: &reqwest::Client,
    request: &LiteChatRequest,
) -> LlmResult<Value> {
    build_anthropic_messages_body_from_litellm_request_with_image_hook(client, request, |_source| {
        ready(None::<String>)
    })
    .await
}

/// Build Anthropic `messages` body from a `litellm-rs` chat request with image hook injection.
///
/// The `image_text_hook` is called for each resolved image source and may return an
/// optional text prefix block to be inserted immediately before that image block.
///
/// # Errors
///
/// Returns an error when image URL parts cannot be resolved into base64 payloads.
#[cfg(feature = "provider-litellm")]
pub async fn build_anthropic_messages_body_from_litellm_request_with_image_hook<F, Fut>(
    client: &reqwest::Client,
    request: &LiteChatRequest,
    mut image_text_hook: F,
) -> LlmResult<Value>
where
    F: FnMut(Base64ImageSource) -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let (system_message, messages) = split_anthropic_system_messages(request.messages.as_slice());
    let anthropic_messages = convert_litellm_messages_to_anthropic_with_image_hook(
        client,
        messages,
        &mut image_text_hook,
    )
    .await?;
    Ok(build_anthropic_messages_body_from_request(
        request,
        anthropic_messages.as_slice(),
        system_message,
    ))
}

/// Execute Anthropic `messages` round-trip from a `litellm-rs` request with default image handling.
///
/// This helper converts request messages, sends HTTP request, and parses Anthropic response blocks.
///
/// # Errors
///
/// Returns an error when request conversion, transport, or response parsing fails.
#[cfg(feature = "provider-litellm")]
pub async fn execute_anthropic_messages_from_litellm_request(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    request: &LiteChatRequest,
    attempts: usize,
) -> LlmResult<AnthropicParsedResponse> {
    execute_anthropic_messages_from_litellm_request_with_image_hook(
        client,
        endpoint,
        api_key,
        request,
        attempts,
        |_source| ready(None::<String>),
    )
    .await
}

/// Execute Anthropic `messages` round-trip from a `litellm-rs` request with image hook injection.
///
/// This helper converts request messages, sends HTTP request, and parses Anthropic response blocks.
///
/// # Errors
///
/// Returns an error when request conversion, transport, or response parsing fails.
#[cfg(feature = "provider-litellm")]
pub async fn execute_anthropic_messages_from_litellm_request_with_image_hook<F, Fut>(
    client: &reqwest::Client,
    endpoint: &str,
    api_key: &str,
    request: &LiteChatRequest,
    attempts: usize,
    image_text_hook: F,
) -> LlmResult<AnthropicParsedResponse>
where
    F: FnMut(Base64ImageSource) -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let body = build_anthropic_messages_body_from_litellm_request_with_image_hook(
        client,
        request,
        image_text_hook,
    )
    .await?;
    let payload =
        send_anthropic_messages_json_with_retry(client, endpoint, api_key, &body, attempts).await?;
    parse_anthropic_messages_response(&payload)
}

/// Split system messages for Anthropic `messages` payload shape.
#[cfg(feature = "provider-litellm")]
#[must_use]
pub fn split_anthropic_system_messages(
    messages: &[LiteChatMessage],
) -> (Option<String>, Vec<LiteChatMessage>) {
    let mut system_parts = Vec::new();
    let mut others = Vec::new();

    for message in messages {
        if matches!(
            message.role,
            LiteMessageRole::System | LiteMessageRole::Developer
        ) {
            if let Some(content) = &message.content {
                match content {
                    LiteMessageContent::Text(text) => system_parts.push(text.clone()),
                    LiteMessageContent::Parts(parts) => {
                        for part in parts {
                            if let LiteContentPart::Text { text } = part {
                                system_parts.push(text.clone());
                            }
                        }
                    }
                }
            }
        } else {
            others.push(message.clone());
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n"))
    };
    (system, others)
}

/// Convert `litellm-rs` chat messages into Anthropic `messages` blocks.
///
/// The `image_text_hook` callback can inject supplemental text before each image.
///
/// # Errors
///
/// Returns an error when an image URL part cannot be resolved into base64 payload.
#[cfg(feature = "provider-litellm")]
pub async fn convert_litellm_messages_to_anthropic_with_image_hook<F, Fut>(
    client: &reqwest::Client,
    messages: Vec<LiteChatMessage>,
    image_text_hook: &mut F,
) -> LlmResult<Vec<Value>>
where
    F: FnMut(Base64ImageSource) -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let mut transformed = Vec::new();
    for message in messages {
        let role = match message.role {
            LiteMessageRole::User | LiteMessageRole::Tool | LiteMessageRole::Function => "user",
            LiteMessageRole::Assistant => "assistant",
            LiteMessageRole::System | LiteMessageRole::Developer => continue,
        };
        let mut content = convert_litellm_message_content_to_anthropic_with_image_hook(
            client,
            message.content,
            image_text_hook,
        )
        .await?;

        if matches!(message.role, LiteMessageRole::Tool)
            && let Some(tool_use_id) = message
                .tool_call_id
                .as_deref()
                .map(str::trim)
                .filter(|id| !id.is_empty())
            && !anthropic_content_contains_tool_result(&content)
        {
            content = json!([{
                "type": "tool_result",
                "tool_use_id": tool_use_id,
                "content": anthropic_tool_result_content(&content),
            }]);
        }

        let mut transformed_message = json!({
            "role": role,
            "content": content,
        });

        if let Some(tool_calls) = message.tool_calls {
            let mut content = Vec::new();
            for call in tool_calls {
                let input = serde_json::from_str::<Value>(&call.function.arguments)
                    .unwrap_or_else(|_| json!({}));
                content.push(json!({
                    "type": "tool_use",
                    "id": call.id,
                    "name": call.function.name,
                    "input": input,
                }));
            }
            transformed_message["content"] = json!(content);
        }

        transformed.push(transformed_message);
    }
    Ok(transformed)
}

#[cfg(feature = "provider-litellm")]
fn anthropic_content_contains_tool_result(content: &Value) -> bool {
    content.as_array().is_some_and(|parts| {
        parts
            .iter()
            .any(|part| part.get("type").and_then(Value::as_str) == Some("tool_result"))
    })
}

#[cfg(feature = "provider-litellm")]
fn anthropic_tool_result_content(content: &Value) -> Value {
    if let Some(text) = content.as_str() {
        return Value::String(text.to_string());
    }
    if let Some(parts) = content.as_array() {
        let text_parts = parts
            .iter()
            .filter_map(|part| {
                if part.get("type").and_then(Value::as_str) == Some("text") {
                    return part.get("text").and_then(Value::as_str);
                }
                None
            })
            .collect::<Vec<_>>();
        if !text_parts.is_empty() {
            return Value::String(text_parts.join("\n"));
        }
    }
    Value::String(content.to_string())
}

#[cfg(feature = "provider-litellm")]
async fn convert_litellm_message_content_to_anthropic_with_image_hook<F, Fut>(
    client: &reqwest::Client,
    content: Option<LiteMessageContent>,
    image_text_hook: &mut F,
) -> LlmResult<Value>
where
    F: FnMut(Base64ImageSource) -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let Some(content) = content else {
        return Ok(json!(""));
    };

    match content {
        LiteMessageContent::Text(text) => Ok(json!(text)),
        LiteMessageContent::Parts(parts) => {
            let mut converted = Vec::new();
            for part in parts {
                match part {
                    LiteContentPart::Text { text } => {
                        converted.push(json!({"type": "text", "text": text}));
                    }
                    LiteContentPart::ImageUrl { image_url } => {
                        let source =
                            resolve_image_source_to_base64(client, image_url.url.as_str()).await?;
                        if let Some(text) = image_text_hook(source.clone()).await {
                            converted.push(json!({"type": "text", "text": text}));
                        }
                        converted.push(anthropic_image_content_part(&source));
                    }
                    LiteContentPart::Image { source, .. } => {
                        let base64_source = Base64ImageSource {
                            media_type: source.media_type.clone(),
                            data: source.data.clone(),
                        };
                        if let Some(text) = image_text_hook(base64_source.clone()).await {
                            converted.push(json!({"type": "text", "text": text}));
                        }
                        converted.push(anthropic_image_content_part(&base64_source));
                    }
                    LiteContentPart::Document { source, .. } => {
                        converted.push(json!({
                            "type": "document",
                            "source": {
                                "type": "base64",
                                "media_type": source.media_type,
                                "data": source.data,
                            }
                        }));
                    }
                    LiteContentPart::ToolResult {
                        tool_use_id,
                        content,
                        is_error,
                    } => {
                        let mut value = json!({
                            "type": "tool_result",
                            "tool_use_id": tool_use_id,
                            "content": content,
                        });
                        if let Some(flag) = is_error {
                            value["is_error"] = json!(flag);
                        }
                        converted.push(value);
                    }
                    LiteContentPart::ToolUse { id, name, input } => {
                        converted.push(json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input,
                        }));
                    }
                    LiteContentPart::Audio { .. } => {}
                }
            }
            Ok(json!(converted))
        }
    }
}

#[cfg(feature = "provider-litellm")]
fn anthropic_image_content_part(source: &Base64ImageSource) -> Value {
    json!({
        "type": "image",
        "source": {
            "type": "base64",
            "media_type": source.media_type.clone(),
            "data": source.data.clone(),
        }
    })
}

#[cfg(feature = "provider-litellm")]
fn normalize_anthropic_messages(messages: &[Value]) -> Vec<Value> {
    messages
        .iter()
        .map(normalize_anthropic_message_image_media_type)
        .collect()
}

#[cfg(feature = "provider-litellm")]
fn normalize_anthropic_message_image_media_type(message: &Value) -> Value {
    let mut normalized = message.clone();
    let Some(content) = normalized.get_mut("content") else {
        return normalized;
    };
    let Some(parts) = content.as_array_mut() else {
        return normalized;
    };

    for part in parts {
        if part.get("type").and_then(Value::as_str) != Some("image") {
            continue;
        }
        let Some(source) = part.get_mut("source").and_then(Value::as_object_mut) else {
            continue;
        };
        let data = source
            .get("data")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let media_type = source
            .get("media_type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        source.insert(
            "media_type".to_string(),
            Value::String(normalize_anthropic_image_media_type(media_type, data)),
        );
    }

    normalized
}

#[cfg(feature = "provider-litellm")]
fn convert_litellm_tools_to_anthropic(
    tools: &[litellm_rs::core::types::tools::Tool],
) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.function.name,
                "description": tool.function.description.clone().unwrap_or_default(),
                "input_schema": tool.function.parameters.clone().unwrap_or_else(|| json!({})),
            })
        })
        .collect()
}

#[cfg(feature = "provider-litellm")]
fn convert_litellm_tool_choice_to_anthropic(tool_choice: &LiteToolChoice) -> Value {
    match tool_choice {
        LiteToolChoice::String(choice) => match choice.as_str() {
            "none" => json!({"type": "none"}),
            "required" => json!({"type": "any"}),
            _ => json!({"type": "auto"}),
        },
        LiteToolChoice::Specific { function, .. } => {
            if let Some(function) = function {
                json!({"type": "tool", "name": function.name})
            } else {
                json!({"type": "auto"})
            }
        }
    }
}

/// Parse Anthropic `messages` response payload into text and tool-use parts.
///
/// # Errors
///
/// Returns `LlmError::Internal` when payload does not include a valid `content` array.
pub fn parse_anthropic_messages_response(payload: &Value) -> LlmResult<AnthropicParsedResponse> {
    let content_items = payload
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| LlmError::Internal {
            message: "anthropic response missing `content` array".to_string(),
        })?;
    let mut text = String::new();
    let mut tool_uses = Vec::new();

    for item in content_items {
        match item.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(part) = item.get("text").and_then(Value::as_str) {
                    text.push_str(part);
                }
            }
            Some("tool_use") => {
                let Some(id) = item.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let Some(name) = item.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let input = item.get("input").cloned().unwrap_or_else(|| json!({}));
                tool_uses.push(AnthropicToolUse {
                    id: id.to_string(),
                    name: name.to_string(),
                    input,
                });
            }
            _ => {}
        }
    }

    Ok(AnthropicParsedResponse {
        text: if text.is_empty() { None } else { Some(text) },
        tool_uses,
    })
}

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
