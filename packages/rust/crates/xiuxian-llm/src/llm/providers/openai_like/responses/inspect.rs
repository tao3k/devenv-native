//! `OpenAI` Responses input diagnostics and tool-chain validation.

use super::payload::normalize_responses_call_id;
use crate::llm::error::{LlmError, LlmResult};
use std::collections::HashSet;

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
