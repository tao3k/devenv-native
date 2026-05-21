//! Codex / `OpenAI` streaming parser.
//!
//! Parses SSE events from OpenAI-compatible APIs, mapping native events
//! to the unified `ZhenfaStreamingEvent` model.

use super::ZhenfaStreamingEvent;
use super::traits::{StreamingOutcome, StreamingTransmuter, TokenUsage};
use serde::Deserialize;

/// `OpenAI` streaming response structure.
#[derive(Debug, Clone, Deserialize)]
struct CodexResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
struct Choice {
    delta: Option<Delta>,
    message: Option<Message>,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Delta {
    content: Option<String>,
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCallDelta>,
}

#[derive(Debug, Clone, Deserialize)]
struct ToolCallDelta {
    index: u32,
    id: Option<String>,
    function: Option<FunctionDelta>,
}

#[derive(Debug, Clone, Deserialize)]
struct FunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Message {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, Deserialize)]
struct ToolCall {
    id: String,
    function: FunctionCall,
}

#[derive(Debug, Clone, Deserialize)]
struct FunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Clone, Deserialize)]
struct Usage {
    #[serde(rename = "prompt_tokens")]
    prompt: u64,
    #[serde(rename = "completion_tokens")]
    completion: u64,
    #[serde(rename = "total_tokens")]
    total: u64,
}

/// Parser for Codex / OpenAI-style streaming output.
#[derive(Debug, Default)]
pub struct CodexStreamingParser {
    accumulated: String,
    tool_call_buffers: Vec<(Option<String>, Option<String>, String)>, // (id, name, args)
    final_usage: Option<TokenUsage>,
    finish_reason: Option<String>,
}

impl CodexStreamingParser {
    /// Create a new Codex streaming parser.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_response_events(&mut self, response: CodexResponse) -> Vec<ZhenfaStreamingEvent> {
        if let Some(usage) = response.usage {
            self.final_usage = Some(TokenUsage {
                input: usage.prompt,
                output: usage.completion,
                total: usage.total,
            });
        }
        response
            .choices
            .iter()
            .flat_map(|choice| self.parse_choice_events(choice))
            .collect()
    }

    fn parse_choice_events(&mut self, choice: &Choice) -> Vec<ZhenfaStreamingEvent> {
        if let Some(reason) = &choice.finish_reason {
            self.finish_reason = Some(reason.clone());
        }

        let mut events = Vec::new();
        if let Some(delta) = &choice.delta {
            self.apply_delta(delta, &mut events);
        }
        if let Some(message) = &choice.message {
            self.apply_message(message, &mut events);
        }
        events
    }

    fn apply_delta(&mut self, delta: &Delta, events: &mut Vec<ZhenfaStreamingEvent>) {
        if let Some(content) = &delta.content {
            self.accumulated.push_str(content);
            events.push(ZhenfaStreamingEvent::TextDelta(content.clone().into()));
        }
        if let Some(reasoning) = &delta.reasoning_content {
            events.push(ZhenfaStreamingEvent::Thought(reasoning.clone().into()));
        }
        delta
            .tool_calls
            .iter()
            .for_each(|tool_call| self.apply_tool_call_delta(tool_call));
    }

    fn apply_tool_call_delta(&mut self, tool_call: &ToolCallDelta) {
        while self.tool_call_buffers.len() <= tool_call.index as usize {
            self.tool_call_buffers.push((None, None, String::new()));
        }

        let buffer = &mut self.tool_call_buffers[tool_call.index as usize];
        if let Some(id) = &tool_call.id {
            buffer.0 = Some(id.clone());
        }
        if let Some(function) = &tool_call.function {
            if let Some(name) = &function.name {
                buffer.1 = Some(name.clone());
            }
            if let Some(args) = &function.arguments {
                buffer.2.push_str(args);
            }
        }
    }

    fn apply_message(&mut self, message: &Message, events: &mut Vec<ZhenfaStreamingEvent>) {
        if let Some(content) = &message.content {
            self.accumulated.push_str(content);
        }
        events.extend(message.tool_calls.iter().map(complete_tool_call_event));
    }
}

impl StreamingTransmuter for CodexStreamingParser {
    fn parse_line(&mut self, line: &str) -> Result<Vec<ZhenfaStreamingEvent>, String> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(Vec::new());
        }

        // Handle SSE format
        let json_str = if let Some(stripped) = line.strip_prefix("data: ") {
            stripped
        } else {
            line
        };

        // Check for stream end
        if json_str == "[DONE]" {
            let outcome = StreamingOutcome {
                success: true,
                tokens_used: self.final_usage,
                final_text: std::mem::take(&mut self.accumulated).into(),
                tool_calls: Vec::new(),
                exit_code: Some(0),
            };
            return Ok(vec![ZhenfaStreamingEvent::Finished(outcome)]);
        }

        let response: CodexResponse = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse Codex response: {e}"))?;

        Ok(self.parse_response_events(response))
    }

    fn finalize(&mut self) -> Result<Option<ZhenfaStreamingEvent>, String> {
        // Emit any pending tool calls
        for (id, name, args) in std::mem::take(&mut self.tool_call_buffers) {
            if let (Some(id), Some(name)) = (id, name) {
                let input: serde_json::Value =
                    serde_json::from_str(&args).unwrap_or(serde_json::Value::Null);
                // Return first tool call, store rest for later
                return Ok(Some(ZhenfaStreamingEvent::ToolCall {
                    id: id.into(),
                    name: name.into(),
                    input,
                }));
            }
        }

        if !self.accumulated.is_empty() || self.final_usage.is_some() {
            let outcome = StreamingOutcome {
                success: true,
                tokens_used: self.final_usage.take(),
                final_text: std::mem::take(&mut self.accumulated).into(),
                tool_calls: Vec::new(),
                exit_code: Some(0),
            };
            Ok(Some(ZhenfaStreamingEvent::Finished(outcome)))
        } else {
            Ok(None)
        }
    }

    fn accumulated_text(&self) -> &str {
        &self.accumulated
    }

    fn reset(&mut self) {
        self.accumulated.clear();
        self.tool_call_buffers.clear();
        self.final_usage = None;
        self.finish_reason = None;
    }

    fn provider_name(&self) -> &'static str {
        "codex"
    }
}

fn complete_tool_call_event(tool_call: &ToolCall) -> ZhenfaStreamingEvent {
    let input: serde_json::Value =
        serde_json::from_str(&tool_call.function.arguments).unwrap_or(serde_json::Value::Null);
    ZhenfaStreamingEvent::ToolCall {
        id: tool_call.id.clone().into(),
        name: tool_call.function.name.clone().into(),
        input,
    }
}
