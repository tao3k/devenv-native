//! `OpenAI` Responses API stream parsing.

use super::payload::{normalize_responses_call_id, remap_openai_responses_tool_name};
use super::types::{
    OpenAiResponsesAssistantOutput, OpenAiResponsesFunctionCall, OpenAiResponsesToolCall,
    OpenAiResponsesToolType,
};
use crate::llm::error::{LlmError, LlmResult};
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

/// Parse an `OpenAI` `/responses` SSE stream body into assistant text and tool calls.
///
/// # Errors
///
/// Returns an error when the stream completes without assistant content or
/// tool calls.
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
