//! Anthropic messages request conversion from `LiteLLM` chat payloads.

#[cfg(feature = "provider-litellm")]
use std::future::{Future, ready};

#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::chat::{
    ChatMessage as LiteChatMessage, ChatRequest as LiteChatRequest,
};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::content::ContentPart as LiteContentPart;
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::message::{
    MessageContent as LiteMessageContent, MessageRole as LiteMessageRole,
};
#[cfg(feature = "provider-litellm")]
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
#[cfg(feature = "provider-litellm")]
use serde_json::{Value, json};

#[cfg(feature = "provider-litellm")]
use crate::llm::LlmResult;
#[cfg(feature = "provider-litellm")]
use crate::llm::multimodal::{Base64ImageSource, ImageMediaType, resolve_image_source_to_base64};

#[cfg(feature = "provider-litellm")]
use super::http::{AnthropicMessagesHttpRequest, send_anthropic_messages_json_with_retry};
#[cfg(feature = "provider-litellm")]
use super::media::normalize_anthropic_image_media_type;
#[cfg(feature = "provider-litellm")]
use super::response::parse_anthropic_messages_response;
#[cfg(feature = "provider-litellm")]
use super::types::AnthropicParsedResponse;

/// Execution input for Anthropic message conversion and transport.
#[cfg(feature = "provider-litellm")]
#[derive(Debug, Clone, Copy)]
pub struct AnthropicMessagesExecutionRequest<'a> {
    /// Shared HTTP client.
    pub client: &'a reqwest::Client,
    /// Provider endpoint URL.
    pub endpoint: &'a str,
    /// Provider API key.
    pub api_key: &'a str,
    /// `LiteLLM` chat request.
    pub request: &'a LiteChatRequest,
    /// Maximum transport attempts.
    pub attempts: usize,
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
    request: AnthropicMessagesExecutionRequest<'_>,
) -> LlmResult<AnthropicParsedResponse> {
    execute_anthropic_messages_from_litellm_request_with_image_hook(request, |_source| {
        ready(None::<String>)
    })
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
    request: AnthropicMessagesExecutionRequest<'_>,
    image_text_hook: F,
) -> LlmResult<AnthropicParsedResponse>
where
    F: FnMut(Base64ImageSource) -> Fut,
    Fut: Future<Output = Option<String>>,
{
    let body = build_anthropic_messages_body_from_litellm_request_with_image_hook(
        request.client,
        request.request,
        image_text_hook,
    )
    .await?;
    let payload = send_anthropic_messages_json_with_retry(AnthropicMessagesHttpRequest {
        client: request.client,
        endpoint: request.endpoint,
        api_key: request.api_key,
        body: &body,
        attempts: request.attempts,
    })
    .await?;
    parse_anthropic_messages_response(&payload)
}

/// Split system messages for Anthropic `messages` payload shape.
#[cfg(feature = "provider-litellm")]
#[must_use]
pub fn split_anthropic_system_messages(
    messages: &[LiteChatMessage],
) -> (Option<String>, Vec<LiteChatMessage>) {
    split_anthropic_system_messages_impl(messages)
}

#[cfg(feature = "provider-litellm")]
fn split_anthropic_system_messages_impl(
    messages: &[LiteChatMessage],
) -> (Option<String>, Vec<LiteChatMessage>) {
    let (system_messages, others): (Vec<_>, Vec<_>) = messages
        .iter()
        .partition(|message| is_system_message(message));
    let system_parts = system_messages
        .into_iter()
        .flat_map(system_message_text_parts)
        .collect::<Vec<_>>();
    let system = if system_parts.is_empty() {
        None
    } else {
        Some(system_parts.join("\n"))
    };
    (system, others.into_iter().cloned().collect())
}

#[cfg(feature = "provider-litellm")]
fn system_message_text_parts(message: &LiteChatMessage) -> Vec<String> {
    match &message.content {
        Some(LiteMessageContent::Text(text)) => vec![text.clone()],
        Some(LiteMessageContent::Parts(parts)) => parts
            .iter()
            .filter_map(|part| match part {
                LiteContentPart::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(feature = "provider-litellm")]
fn is_system_message(message: &LiteChatMessage) -> bool {
    matches!(
        message.role,
        LiteMessageRole::System | LiteMessageRole::Developer
    )
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
    convert_litellm_messages_to_anthropic_with_image_hook_impl(client, messages, image_text_hook)
        .await
}

#[cfg(feature = "provider-litellm")]
async fn convert_litellm_messages_to_anthropic_with_image_hook_impl<F, Fut>(
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
                            media_type: ImageMediaType::new(source.media_type.clone()),
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
            "media_type": source.media_type.to_string(),
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
