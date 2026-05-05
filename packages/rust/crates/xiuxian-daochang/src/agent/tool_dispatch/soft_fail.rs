use super::execution::{degraded_tool_error_payload, is_timeout_error_message};
use super::tool_types::ToolCallOutput;
use crate::Agent;

const MEMORY_SEARCH_TOOL_NAME: &str = "memory.search_memory";
const MEMORY_SAVE_TOOL_NAME: &str = "memory.save_memory";

impl Agent {
    pub(in crate::agent) fn soft_fail_tool_error_output(
        name: &str,
        tool_call_id: Option<&str>,
        error: &anyhow::Error,
    ) -> Option<ToolCallOutput> {
        soft_fail_tool_error_output(name, tool_call_id, error)
    }
}

fn soft_fail_tool_error_output(
    name: &str,
    tool_call_id: Option<&str>,
    error: &anyhow::Error,
) -> Option<ToolCallOutput> {
    let message = format!("{error:#}");
    let lower = message.to_ascii_lowercase();
    if name == MEMORY_SAVE_TOOL_NAME {
        let error_kind = if is_timeout_error_message(&lower) {
            "timeout"
        } else {
            "save_failed"
        };
        tracing::warn!(
            event = "agent.tool_dispatch.soft_fail",
            tool = name,
            error_kind,
            error = %message,
            "external tool failed while saving memory; degrading to soft tool error output"
        );
        return Some(ToolCallOutput::error(
            degraded_tool_error_payload(
                name,
                None,
                error_kind,
                None,
                "Memory save failed; continuing without blocking this turn.",
            ),
            tool_call_id,
        ));
    }

    let is_embedding_timeout = lower.contains("embedding timed out")
        || (lower.contains("embedding")
            && lower.contains("timed out")
            && lower.contains("tool runtime error: -32603"));
    if name != MEMORY_SEARCH_TOOL_NAME || !is_embedding_timeout {
        return None;
    }
    tracing::warn!(
        event = "agent.tool_dispatch.soft_fail",
        tool = name,
        error = %message,
        "external tool failed with embedding timeout; degrading to soft tool error output"
    );
    Some(ToolCallOutput::error(
        degraded_tool_error_payload(
            name,
            None,
            "embedding_timeout",
            None,
            "Embedding lookup timed out; continuing without tool result.",
        ),
        tool_call_id,
    ))
}
