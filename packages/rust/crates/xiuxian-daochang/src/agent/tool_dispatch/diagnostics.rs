use super::execution::timeout_tool_error_payload;
use super::tool_types::ToolCallOutput;

pub(super) fn log_tool_dispatch_success(source: &str, name: &str, session_id: Option<&str>) {
    tracing::info!(
        event = "agent.tool.dispatch",
        source,
        tool = name,
        session_id = session_id.unwrap_or(""),
        outcome = "success",
        is_error = false,
        "tool dispatch completed"
    );
}

pub(super) fn log_tool_dispatch_error_with_detail(
    source: &str,
    name: &str,
    session_id: Option<&str>,
    outcome: &str,
    error: &impl std::fmt::Display,
    message: &str,
) {
    tracing::warn!(
        event = "agent.tool.dispatch",
        source,
        tool = name,
        session_id = session_id.unwrap_or(""),
        outcome,
        is_error = true,
        error = %error,
        "{message}"
    );
}

pub(super) fn log_tool_dispatch_error(
    source: &str,
    name: &str,
    session_id: Option<&str>,
    message: &str,
) {
    tracing::warn!(
        event = "agent.tool.dispatch",
        source,
        tool = name,
        session_id = session_id.unwrap_or(""),
        outcome = "error",
        is_error = true,
        "{message}"
    );
}

pub(super) fn log_tool_dispatch_timeout(
    source: &str,
    name: &str,
    session_id: Option<&str>,
    timeout_secs: u64,
) {
    tracing::warn!(
        event = "agent.tool.dispatch",
        source,
        tool = name,
        session_id = session_id.unwrap_or(""),
        outcome = "timeout",
        is_error = true,
        timeout_secs,
        "tool dispatch timed out and was degraded to non-fatal tool error"
    );
}

pub(super) fn log_tool_dispatch_timeout_with_detail(
    source: &str,
    name: &str,
    session_id: Option<&str>,
    timeout_secs: u64,
    error: &str,
) {
    tracing::warn!(
        event = "agent.tool.dispatch",
        source,
        tool = name,
        session_id = session_id.unwrap_or(""),
        outcome = "timeout",
        is_error = true,
        timeout_secs,
        error = %error,
        "tool dispatch timed out and was degraded to non-fatal tool error"
    );
}

pub(super) fn tool_timeout_error_output(
    source: &str,
    name: &str,
    timeout_secs: u64,
    tool_call_id: Option<&str>,
) -> ToolCallOutput {
    ToolCallOutput::error(
        timeout_tool_error_payload(source, name, timeout_secs),
        tool_call_id,
    )
}
