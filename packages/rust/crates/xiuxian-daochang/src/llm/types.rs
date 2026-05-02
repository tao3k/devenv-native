//! LLM message and tool-call data types.

use serde::{Deserialize, Serialize};

use crate::session::ToolCallOut;

/// Request body for chat completions (`OpenAI` format).
#[derive(Debug, Serialize)]
pub(super) struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<crate::session::ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ToolDef {
    #[serde(rename = "type")]
    pub typ: String,
    pub function: FunctionDef,
}

#[derive(Debug, Serialize)]
pub(super) struct FunctionDef {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

/// Response: choices[0].message.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
/// One chat completion choice.
pub struct Choice {
    pub message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
/// Assistant message returned by the LLM provider.
pub struct AssistantMessage {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCallOut>>,
}
