//! `OpenAI` Responses API payload building and stream parsing.

use crate::llm::error::{LlmError, LlmResult};
use litellm_rs::core::types::chat::{
    ChatMessage as LiteChatMessage, ChatRequest as LiteChatRequest,
};
use litellm_rs::core::types::content::ContentPart;
use litellm_rs::core::types::message::{MessageContent, MessageRole};
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

const RESPONSES_TOOL_NAME_FALLBACK: &str = "tool";

/// Prepared `OpenAI` `/responses` payload plus tool-alias mapping metadata.
#[derive(Debug, Clone)]
pub struct OpenAiResponsesPayload {
    /// Serialized request body.
    pub payload: serde_json::Value,
    /// Reverse mapping for normalized tool aliases.
    pub alias_to_original_tool_name: HashMap<String, String>,
}

/// Parsed function-call payload from an `OpenAI` `/responses` stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiResponsesFunctionCall {
    /// Function/tool name (after alias remapping).
    pub name: String,
    /// JSON-serialized function arguments.
    pub arguments: String,
}

/// Parsed tool-call item from an `OpenAI` `/responses` stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiResponsesToolCall {
    /// Stable call identifier.
    pub id: String,
    /// Tool type reported by provider (defaults to `function`).
    pub tool_type: OpenAiResponsesToolType,
    /// Function invocation payload.
    pub function: OpenAiResponsesFunctionCall,
}

/// Tool type reported by the `OpenAI` Responses stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiResponsesToolType(String);

impl OpenAiResponsesToolType {
    /// Creates a response tool type label.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw provider label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq<&str> for OpenAiResponsesToolType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl std::fmt::Display for OpenAiResponsesToolType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Parsed assistant output reconstructed from an `OpenAI` `/responses` stream body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiResponsesAssistantOutput {
    /// Consolidated assistant text output.
    pub content: Option<String>,
    /// Parsed tool calls emitted by the assistant.
    pub tool_calls: Vec<OpenAiResponsesToolCall>,
}

/// Build an `OpenAI` `/responses` payload with tool-name normalization.
#[must_use]
pub fn build_openai_responses_payload(request: &LiteChatRequest) -> OpenAiResponsesPayload {
    let mut alias_to_original_tool_name = HashMap::new();
    let mut original_to_alias_tool_name = HashMap::new();
    let mut used_aliases = HashSet::new();
    let tools_payload = request.tools.as_ref().and_then(|tools| {
        (!tools.is_empty()).then(|| {
            tools
                .iter()
                .map(|tool| {
                    let original_name = &tool.function.name;
                    let alias_name = reserve_responses_tool_name_alias(
                        original_name,
                        &mut original_to_alias_tool_name,
                        &mut alias_to_original_tool_name,
                        &mut used_aliases,
                    );
                    serde_json::json!({
                        "type": "function",
                        "name": alias_name,
                        "description": tool.function.description,
                        "parameters": normalize_responses_tool_parameters(tool.function.parameters.as_ref()),
                    })
                })
                .collect::<Vec<_>>()
        })
    });
    let mut payload = serde_json::json!({
        "model": request.model,
        "stream": true,
        "input": to_responses_input(&request.messages, &original_to_alias_tool_name),
    });
    if let Some(max_tokens) = request.max_tokens {
        payload["max_output_tokens"] = serde_json::json!(max_tokens);
    }
    if let Some(tools) = tools_payload {
        payload["tools"] = serde_json::Value::Array(tools);
    }
    if let Some(tool_choice) = &request.tool_choice {
        payload["tool_choice"] =
            serialize_responses_tool_choice(tool_choice, &original_to_alias_tool_name);
    }
    OpenAiResponsesPayload {
        payload,
        alias_to_original_tool_name,
    }
}

/// Resolve possibly-normalized `/responses` tool name back to original registered tool name.
#[must_use]
pub fn remap_openai_responses_tool_name(
    name: &str,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) -> String {
    alias_to_original_tool_name
        .get(name)
        .cloned()
        .unwrap_or_else(|| name.to_string())
}

fn normalize_responses_call_id(raw_id: &str) -> Option<&str> {
    raw_id
        .split('|')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn to_responses_input(
    messages: &[LiteChatMessage],
    original_to_alias_tool_name: &HashMap<String, String, impl BuildHasher>,
) -> Vec<serde_json::Value> {
    let mut input = Vec::new();
    let mut generated_legacy_call_id = 0usize;

    for message in messages {
        if message.role == MessageRole::Assistant {
            append_assistant_tool_call_items(
                &mut input,
                message,
                original_to_alias_tool_name,
                &mut generated_legacy_call_id,
            );
        }

        if message.role == MessageRole::Tool {
            let Some(call_id) = message
                .tool_call_id
                .as_deref()
                .and_then(normalize_responses_call_id)
            else {
                continue;
            };
            input.push(serde_json::json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": content_to_plain_text(message.content.as_ref()),
            }));
            continue;
        }

        let content = content_to_responses_content(message.content.as_ref());
        if responses_content_is_empty(&content) {
            continue;
        }
        input.push(serde_json::json!({
            "role": message.role.to_string(),
            "content": content,
        }));
    }

    input
}

fn append_assistant_tool_call_items(
    input: &mut Vec<serde_json::Value>,
    message: &LiteChatMessage,
    original_to_alias_tool_name: &HashMap<String, String, impl BuildHasher>,
    generated_legacy_call_id: &mut usize,
) {
    if let Some(tool_calls) = message
        .tool_calls
        .as_ref()
        .filter(|tool_calls| !tool_calls.is_empty())
    {
        for tool_call in tool_calls {
            let Some(call_id) = normalize_responses_call_id(tool_call.id.as_str()) else {
                continue;
            };
            input.push(serde_json::json!({
                "type": "function_call",
                "call_id": call_id,
                "name": alias_responses_function_name(
                    tool_call.function.name.as_str(),
                    original_to_alias_tool_name,
                ),
                "arguments": normalize_function_call_arguments(tool_call.function.arguments.as_str()),
            }));
        }
        return;
    }

    let Some(function_call) = &message.function_call else {
        return;
    };
    let call_id = message
        .tool_call_id
        .as_deref()
        .and_then(normalize_responses_call_id)
        .map_or_else(
            || {
                *generated_legacy_call_id = generated_legacy_call_id.saturating_add(1);
                format!("call_legacy_{generated_legacy_call_id}")
            },
            std::borrow::ToOwned::to_owned,
        );

    input.push(serde_json::json!({
        "type": "function_call",
        "call_id": call_id,
        "name": alias_responses_function_name(function_call.name.as_str(), original_to_alias_tool_name),
        "arguments": normalize_function_call_arguments(function_call.arguments.as_str()),
    }));
}

fn alias_responses_function_name(
    function_name: &str,
    original_to_alias_tool_name: &HashMap<String, String, impl BuildHasher>,
) -> String {
    original_to_alias_tool_name
        .get(function_name)
        .cloned()
        .unwrap_or_else(|| sanitize_responses_tool_name(function_name))
}

fn normalize_function_call_arguments(arguments: &str) -> String {
    let trimmed = arguments.trim();
    if trimmed.is_empty() {
        "{}".to_string()
    } else {
        trimmed.to_string()
    }
}

fn responses_content_is_empty(content: &serde_json::Value) -> bool {
    match content {
        serde_json::Value::Null => true,
        serde_json::Value::String(text) => text.trim().is_empty(),
        serde_json::Value::Array(items) => items.is_empty(),
        serde_json::Value::Object(object) => object.is_empty(),
        _ => false,
    }
}

fn content_to_responses_content(content: Option<&MessageContent>) -> serde_json::Value {
    let Some(content) = content else {
        return serde_json::Value::String(String::new());
    };
    match content {
        MessageContent::Text(text) => serde_json::Value::String(text.clone()),
        MessageContent::Parts(parts) => serde_json::json!(
            parts
                .iter()
                .filter_map(content_part_to_responses_input_part)
                .collect::<Vec<_>>()
        ),
    }
}

fn content_part_to_responses_input_part(part: &ContentPart) -> Option<serde_json::Value> {
    match part {
        ContentPart::Text { text } => Some(serde_json::json!({
            "type": "input_text",
            "text": text
        })),
        ContentPart::ImageUrl { image_url } => Some(serde_json::json!({
            "type": "input_image",
            "image_url": image_url.url,
            "detail": image_url.detail.clone().unwrap_or_else(|| "auto".to_string()),
        })),
        ContentPart::Image {
            source,
            detail,
            image_url,
        } => {
            let uri = if let Some(image_url) = image_url {
                image_url.url.clone()
            } else {
                format!("data:{};base64,{}", source.media_type, source.data)
            };
            Some(serde_json::json!({
                "type": "input_image",
                "image_url": uri,
                "detail": detail.clone().unwrap_or_else(|| "auto".to_string()),
            }))
        }
        _ => None,
    }
}

fn content_to_plain_text(content: Option<&MessageContent>) -> String {
    let Some(content) = content else {
        return String::new();
    };
    match content {
        MessageContent::Text(text) => text.clone(),
        MessageContent::Parts(parts) => parts
            .iter()
            .filter_map(content_part_to_plain_text)
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn content_part_to_plain_text(part: &ContentPart) -> Option<String> {
    match part {
        ContentPart::Text { text } => Some(text.clone()),
        ContentPart::ToolResult { content, .. } => match content {
            serde_json::Value::Null => None,
            serde_json::Value::String(text) => Some(text.clone()),
            other => Some(other.to_string()),
        },
        _ => None,
    }
}

#[must_use]
pub(crate) fn summarize_openai_responses_input(payload: &serde_json::Value) -> String {
    let Some(input) = payload.get("input").and_then(serde_json::Value::as_array) else {
        return "<no-input-array>".to_string();
    };
    if input.is_empty() {
        return "<empty-input-array>".to_string();
    }

    input
        .iter()
        .enumerate()
        .map(|(index, item)| summarize_openai_responses_input_item(index, item))
        .collect::<Vec<_>>()
        .join(" | ")
}

pub(crate) fn validate_openai_responses_input_tool_chain(
    payload: &serde_json::Value,
) -> LlmResult<()> {
    let Some(input) = payload.get("input").and_then(serde_json::Value::as_array) else {
        return Ok(());
    };

    let mut seen_function_calls = HashSet::new();
    let mut open_function_calls = HashSet::new();
    let mut duplicate_function_calls = Vec::new();
    let mut unmatched_outputs = Vec::new();
    let summary = summarize_openai_responses_input(payload);

    for (index, item) in input.iter().enumerate() {
        let item_type = item_type_for_openai_responses_input_item(item);
        let call_id = item
            .get("call_id")
            .and_then(serde_json::Value::as_str)
            .and_then(normalize_responses_call_id);

        match item_type {
            "function_call" => {
                if let Some(call_id) = call_id {
                    let owned = call_id.to_string();
                    if !seen_function_calls.insert(owned.clone()) {
                        duplicate_function_calls.push(format!("{index}:{call_id}"));
                    }
                    open_function_calls.insert(owned);
                }
            }
            "function_call_output" => {
                if let Some(call_id) = call_id
                    && !open_function_calls.remove(call_id)
                {
                    unmatched_outputs.push(format!("{index}:{call_id}"));
                }
            }
            _ => {}
        }
    }

    if duplicate_function_calls.is_empty() && unmatched_outputs.is_empty() {
        return Ok(());
    }

    let mut problems = Vec::new();
    if !duplicate_function_calls.is_empty() {
        problems.push(format!(
            "duplicate function_call ids: {}",
            duplicate_function_calls.join(", ")
        ));
    }
    if !unmatched_outputs.is_empty() {
        problems.push(format!(
            "function_call_output items without an available preceding function_call: {}",
            unmatched_outputs.join(", ")
        ));
    }

    Err(LlmError::Internal {
        message: format!(
            "OpenAI /responses payload contains invalid tool-call chain: {}; input_summary={summary}",
            problems.join("; ")
        ),
    })
}

fn summarize_openai_responses_input_item(index: usize, item: &serde_json::Value) -> String {
    let item_type = item_type_for_openai_responses_input_item(item);
    let call_id = item
        .get("call_id")
        .and_then(serde_json::Value::as_str)
        .and_then(normalize_responses_call_id);
    let name = item
        .get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match (call_id, name) {
        (Some(call_id), Some(name)) => format!("{index}:{item_type}({call_id},{name})"),
        (Some(call_id), None) => format!("{index}:{item_type}({call_id})"),
        (None, Some(name)) => format!("{index}:{item_type}({name})"),
        (None, None) => format!("{index}:{item_type}"),
    }
}

fn item_type_for_openai_responses_input_item(item: &serde_json::Value) -> &str {
    item.get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| {
            item.get("role")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown")
        })
}

fn reserve_responses_tool_name_alias(
    original_name: &str,
    original_to_alias_tool_name: &mut HashMap<String, String>,
    alias_to_original_tool_name: &mut HashMap<String, String>,
    used_aliases: &mut HashSet<String>,
) -> String {
    if let Some(alias) = original_to_alias_tool_name.get(original_name) {
        return alias.clone();
    }

    let base_alias = sanitize_responses_tool_name(original_name);
    let mut alias = base_alias.clone();
    let mut collision_suffix = 2u32;

    while !used_aliases.insert(alias.clone()) {
        alias = format!("{base_alias}_{collision_suffix}");
        collision_suffix = collision_suffix.saturating_add(1);
    }

    original_to_alias_tool_name.insert(original_name.to_string(), alias.clone());
    alias_to_original_tool_name.insert(alias.clone(), original_name.to_string());
    alias
}

fn sanitize_responses_tool_name(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        RESPONSES_TOOL_NAME_FALLBACK.to_string()
    } else {
        sanitized
    }
}

fn serialize_responses_tool_choice(
    tool_choice: &LiteToolChoice,
    original_to_alias_tool_name: &HashMap<String, String>,
) -> serde_json::Value {
    let mut value = serde_json::to_value(tool_choice).unwrap_or(serde_json::Value::Null);
    remap_responses_tool_choice_name(&mut value, original_to_alias_tool_name);
    value
}

fn remap_responses_tool_choice_name(
    value: &mut serde_json::Value,
    original_to_alias_tool_name: &HashMap<String, String>,
) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                remap_responses_tool_choice_name(item, original_to_alias_tool_name);
            }
        }
        serde_json::Value::Object(object) => {
            if let Some(name) = object.get("name").and_then(serde_json::Value::as_str)
                && let Some(alias) = original_to_alias_tool_name.get(name)
            {
                object.insert("name".to_string(), serde_json::Value::String(alias.clone()));
            }
            if let Some(function_value) = object.get_mut("function") {
                remap_responses_tool_choice_name(function_value, original_to_alias_tool_name);
            }
        }
        _ => {}
    }
}

fn normalize_responses_tool_parameters(
    parameters: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut schema = parameters.cloned().unwrap_or_else(|| {
        serde_json::json!({
            "type": "object",
            "properties": {},
        })
    });

    let Some(object) = schema.as_object_mut() else {
        return serde_json::json!({
            "type": "object",
            "properties": {},
        });
    };

    if !matches!(
        object.get("type").and_then(serde_json::Value::as_str),
        Some("object")
    ) {
        return schema;
    }

    if !matches!(object.get("properties"), Some(serde_json::Value::Object(_))) {
        object.insert("properties".to_string(), serde_json::json!({}));
    }

    schema
}

/// Parse an `OpenAI` `/responses` stream body into assistant text and tool calls.
///
/// # Errors
///
/// Returns an error when the stream yields neither text nor tool calls.
pub fn parse_openai_responses_stream(
    raw: &str,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) -> LlmResult<OpenAiResponsesAssistantOutput> {
    parse_openai_responses_stream_impl(raw, alias_to_original_tool_name)
}

fn parse_openai_responses_stream_impl(
    raw: &str,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) -> LlmResult<OpenAiResponsesAssistantOutput> {
    let mut text_deltas = String::new();
    let mut output_text_done = String::new();
    let mut item_message_text = String::new();
    let mut tool_calls = Vec::new();
    let mut seen_tool_ids = HashSet::new();
    let mut seen_message_keys = HashSet::new();

    let mut state = ResponsesStreamParseState {
        text_deltas: &mut text_deltas,
        output_text_done: &mut output_text_done,
        item_message_text: &mut item_message_text,
        tool_calls: &mut tool_calls,
        seen_tool_ids: &mut seen_tool_ids,
        seen_message_keys: &mut seen_message_keys,
    };

    for event in raw.lines().filter_map(parse_responses_sse_event) {
        apply_responses_stream_event(&event, &mut state, alias_to_original_tool_name);
    }

    let content = if !item_message_text.trim().is_empty() {
        Some(item_message_text.trim().to_string())
    } else if !output_text_done.trim().is_empty() {
        Some(output_text_done.trim().to_string())
    } else if !text_deltas.trim().is_empty() {
        Some(text_deltas.trim().to_string())
    } else {
        None
    };

    if content.is_none() && tool_calls.is_empty() {
        return Err(LlmError::Internal {
            message: "responses stream completed without content or tool calls".to_string(),
        });
    }

    Ok(OpenAiResponsesAssistantOutput {
        content,
        tool_calls,
    })
}

struct ResponsesStreamParseState<'a> {
    text_deltas: &'a mut String,
    output_text_done: &'a mut String,
    item_message_text: &'a mut String,
    tool_calls: &'a mut Vec<OpenAiResponsesToolCall>,
    seen_tool_ids: &'a mut HashSet<String>,
    seen_message_keys: &'a mut HashSet<String>,
}

fn parse_responses_sse_event(line: &str) -> Option<serde_json::Value> {
    let trimmed = line.trim();
    if !trimmed.starts_with("data:") {
        return None;
    }
    let payload = trimmed.trim_start_matches("data:").trim();
    if payload.is_empty() || payload == "[DONE]" {
        return None;
    }
    serde_json::from_str(payload).ok()
}

fn apply_responses_stream_event(
    event: &serde_json::Value,
    state: &mut ResponsesStreamParseState<'_>,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) {
    match responses_event_type(event) {
        "response.output_text.delta" | "response.text.delta" => {
            append_event_text(event, "delta", state.text_deltas);
        }
        "response.output_text.done" | "response.text.done" => {
            append_event_text(event, "text", state.output_text_done);
        }
        "response.output_item.done" => {
            collect_event_item(event, state, alias_to_original_tool_name);
        }
        "response.completed" => {
            collect_completed_response(event, state, alias_to_original_tool_name);
        }
        _ => {}
    }
}

fn responses_event_type(event: &serde_json::Value) -> &str {
    event
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
}

fn append_event_text(event: &serde_json::Value, field: &str, target: &mut String) {
    if let Some(text) = event.get(field).and_then(serde_json::Value::as_str) {
        target.push_str(text);
    }
}

fn collect_event_item(
    event: &serde_json::Value,
    state: &mut ResponsesStreamParseState<'_>,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) {
    if let Some(item) = event.get("item") {
        collect_parsed_responses_item(
            item,
            state.item_message_text,
            state.tool_calls,
            state.seen_tool_ids,
            state.seen_message_keys,
            alias_to_original_tool_name,
        );
    }
}

fn collect_completed_response(
    event: &serde_json::Value,
    state: &mut ResponsesStreamParseState<'_>,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) {
    let Some(output) = event
        .get("response")
        .and_then(|response| response.get("output"))
        .and_then(serde_json::Value::as_array)
    else {
        return;
    };
    for item in output {
        collect_parsed_responses_item(
            item,
            state.item_message_text,
            state.tool_calls,
            state.seen_tool_ids,
            state.seen_message_keys,
            alias_to_original_tool_name,
        );
    }
}

fn collect_parsed_responses_item(
    item: &serde_json::Value,
    final_text: &mut String,
    tool_calls: &mut Vec<OpenAiResponsesToolCall>,
    seen_tool_ids: &mut HashSet<String>,
    seen_message_keys: &mut HashSet<String>,
    alias_to_original_tool_name: &HashMap<String, String, impl BuildHasher>,
) {
    let item_type = item
        .get("type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    match item_type {
        "message" => {
            let mut message_text_parts = Vec::new();
            if let Some(content) = item.get("content").and_then(serde_json::Value::as_array) {
                for part in content {
                    if part.get("type").and_then(serde_json::Value::as_str) != Some("output_text") {
                        continue;
                    }
                    if let Some(text) = part.get("text").and_then(serde_json::Value::as_str)
                        && !text.trim().is_empty()
                    {
                        message_text_parts.push(text.trim().to_string());
                    }
                }
            }
            if message_text_parts.is_empty() {
                return;
            }
            let dedup_key = item
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map_or_else(
                    || format!("text:{}", message_text_parts.join("\n")),
                    std::borrow::ToOwned::to_owned,
                );
            if !seen_message_keys.insert(dedup_key) {
                return;
            }
            if !final_text.is_empty() {
                final_text.push('\n');
            }
            final_text.push_str(message_text_parts.join("\n").as_str());
        }
        "function_call" => {
            let call_id = item
                .get("call_id")
                .and_then(serde_json::Value::as_str)
                .or_else(|| item.get("id").and_then(serde_json::Value::as_str))
                .and_then(normalize_responses_call_id)
                .unwrap_or("call_0")
                .to_string();
            if seen_tool_ids.contains(&call_id) {
                return;
            }
            let name = item
                .get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if name.is_empty() {
                return;
            }
            let arguments = item
                .get("arguments")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("{}")
                .to_string();
            tool_calls.push(OpenAiResponsesToolCall {
                id: call_id.clone(),
                tool_type: OpenAiResponsesToolType::new("function"),
                function: OpenAiResponsesFunctionCall {
                    name: remap_openai_responses_tool_name(name, alias_to_original_tool_name),
                    arguments,
                },
            });
            seen_tool_ids.insert(call_id);
        }
        _ => {}
    }
}
