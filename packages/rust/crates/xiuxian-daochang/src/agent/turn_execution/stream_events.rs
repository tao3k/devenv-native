//! Typed events yielded by [`Agent::run_turn_stream`].

use crate::session::ToolCallOut;

/// Events yielded by a streaming agent turn.
///
/// Consumers receive these through a `tokio::sync::mpsc::Receiver` and
/// translate them to their wire format (SSE, markdown, etc.). The Agent
/// itself is protocol-agnostic.
#[derive(Debug, Clone)]
pub enum AgentStreamEvent {
    /// An incremental text chunk from the assistant's response.
    TextDelta(String),

    /// The assistant requested one or more tool calls.
    /// Emitted once per tool-call batch (one LLM round may produce
    /// multiple tool calls).
    ToolCallsStarted {
        /// The tool calls requested by the model.
        tool_calls: Vec<ToolCallOut>,
    },

    /// A single tool execution completed.
    ToolResult {
        /// The tool call ID this result corresponds to.
        tool_call_id: String,
        /// Tool name.
        name: String,
        /// Serialized tool output.
        output: String,
        /// Whether the tool execution errored.
        is_error: bool,
    },

    /// The turn completed successfully.
    TurnComplete {
        /// The final assembled assistant output.
        final_output: String,
    },

    /// The turn ended with an error.
    TurnError {
        /// Error description.
        error: String,
    },
}
