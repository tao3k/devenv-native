//! `OpenAI` Responses API payload construction.

use super::types::OpenAiResponsesPayload;
use litellm_rs::core::types::chat::{
    ChatMessage as LiteChatMessage, ChatRequest as LiteChatRequest,
};
use litellm_rs::core::types::content::ContentPart;
use litellm_rs::core::types::message::{MessageContent, MessageRole};
use litellm_rs::core::types::tools::ToolChoice as LiteToolChoice;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

const RESPONSES_TOOL_NAME_FALLBACK: &str = "tool";

/// Build an `OpenAI` `/responses` request payload from the shared chat request shape.
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

pub(crate) fn normalize_responses_call_id(raw_id: &str) -> Option<&str> {
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
