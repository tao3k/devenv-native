//! `OpenAI` Responses API data types.

use std::collections::HashMap;

/// Serialized request payload plus tool-name alias metadata for `/responses`.
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
