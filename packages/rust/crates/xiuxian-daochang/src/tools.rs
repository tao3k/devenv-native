//! Tool name qualification for multiple external tool servers.

/// Format: `tool__{server}__{tool}` so the agent can route tool calls to the right external tool server.
#[must_use]
pub fn qualify_tool_name(server: &str, tool: &str) -> String {
    format!("tool__{server}__{tool}")
}

/// Parse a qualified name; returns `Some((server, tool))` or `None` if invalid.
#[must_use]
pub fn parse_qualified_tool_name(qualified: &str) -> Option<(String, String)> {
    let rest = qualified.strip_prefix("tool__")?;
    let (server, tool) = rest.split_once("__")?;
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    Some((server.to_string(), tool.to_string()))
}
