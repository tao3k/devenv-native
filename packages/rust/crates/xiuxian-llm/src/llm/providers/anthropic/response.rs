//! Anthropic messages response parsing.

use serde_json::{Value, json};

use crate::llm::{LlmError, LlmResult};

use super::types::{AnthropicParsedResponse, AnthropicToolUse};

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
